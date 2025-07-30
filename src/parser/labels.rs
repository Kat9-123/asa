use std::collections::HashMap;

use crate::asm_details;
use crate::feedback::*;
use crate::hint;
use crate::println_debug;
use crate::tokens;
use crate::tokens::*;
use colored::Colorize;

pub fn grab_braced_label_definitions(tokens: Vec<Token>) -> Vec<Token> {
    let mut updated_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;

    while i < tokens.len() {
        if let TokenVariant::BraceOpen = &tokens[i].variant {
            let name = match &tokens[i + 1].variant {
                TokenVariant::Label { name } => name,
                _ => todo!(),
            };
            let data: IntOrString = match &tokens[i + 3].variant {
                TokenVariant::DecLiteral { value } => IntOrString::Int(*value),
                TokenVariant::Label { name } => IntOrString::Str(name.clone()),
                _ => todo!(),
            };

            updated_tokens.push(Token {
                info: tokens[i + 3].info.clone(), // take the info of the value pointed at
                variant: TokenVariant::BracedLabelDefinition {
                    name: name.clone(),
                    data,
                },
                origin_info: tokens[i + 3].origin_info.clone(),
            });
            i += 5;
            continue;
        }
        updated_tokens.push(tokens[i].clone());
        i += 1;
    }

    updated_tokens
}

pub fn assign_addresses_to_labels(tokens: &Vec<Token>) -> Vec<HashMap<String, (i32, Info)>> {
    let mut scopes: Vec<HashMap<String, (i32, Info)>> = vec![HashMap::new()];
    let mut address: i32 = 0;
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for token in tokens {
        match &token.variant {
            TokenVariant::Scope => {
                scopes.push(HashMap::new());
                let current_scope_idx = seen_scopes_count + 1;
                current_scope_indexes.push(current_scope_idx);
                println_debug!("SCOPE {:?}", current_scope_indexes);
                seen_scopes_count += 1;
            }
            TokenVariant::Unscope => {
                current_scope_indexes.pop();
            }

            TokenVariant::BracedLabelDefinition { name, data } => {
                if let Some(x) =
                    scopes[current_scope_indexes[current_scope_indexes.len() - 1]].get(name)
                {
                    asm_warn!(
                        &token.info,
                        "The label called '{name}' has already been defined in this scope"
                    );
                    asm_details!(&x.1, "Here");
                }

                scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                    .insert(name.clone(), (address, token.info.clone()));
            }

            TokenVariant::LabelDefinition { name, offset } => {
                if let Some(x) =
                    scopes[current_scope_indexes[current_scope_indexes.len() - 1]].get(name)
                {
                    asm_warn!(
                        &token.info,
                        "The label called '{name}' has already been defined in this scope"
                    );
                    asm_details!(&x.1, "Here");
                }

                scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                    .insert(name.clone(), (address + offset, token.info.clone()));
            }

            _ => {}
        }
        address += token.size();
    }

    println_debug!("{:?}", scopes);
    scopes
}

pub fn resolve_labels(
    tokens: &Vec<Token>,
    scoped_label_table: &Vec<HashMap<String, (i32, Info)>>,
) -> Vec<Token> {
    let mut updated_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for token in tokens {
        match &token.variant {
            TokenVariant::Scope => {
                let current_scope_idx = seen_scopes_count + 1;
                current_scope_indexes.push(current_scope_idx);
                seen_scopes_count += 1;
            }
            TokenVariant::Unscope => {
                current_scope_indexes.pop();
            }
            TokenVariant::Label { name } => {
                let (val, inf) = find_label(
                    name,
                    scoped_label_table,
                    &current_scope_indexes,
                    &token.info,
                );
                updated_tokens.push(Token {
                    info: token.info.clone(),
                    variant: TokenVariant::DecLiteral { value: val },
                    origin_info: token.origin_info.clone(),
                });
            }
            TokenVariant::BracedLabelDefinition { name, data } => match data {
                IntOrString::Int(x) => updated_tokens.push(Token {
                    info: token.info.clone(),
                    variant: TokenVariant::DecLiteral { value: *x },
                    origin_info: token.origin_info.clone(),
                }),
                IntOrString::Str(string) => {
                    let (val, inf) = find_label(
                        name,
                        scoped_label_table,
                        &current_scope_indexes,
                        &token.info,
                    );
                    updated_tokens.push(Token {
                        info: token.info.clone(),

                        variant: TokenVariant::DecLiteral { value: val },
                        origin_info: token.origin_info.clone(),
                    });
                }
            },
            _ => updated_tokens.push(token.clone()),
        }
    }
    updated_tokens
}

fn find_label(
    name: &String,
    scoped_label_table: &Vec<HashMap<String, (i32, Info)>>,
    current_scope_indexes: &Vec<usize>,
    info: &Info,
) -> (i32, Info) {
    for scope in current_scope_indexes.iter().rev() {
        if let Some(x) = scoped_label_table[*scope].get(name) {
            return x.clone();
        }
    }
    if name == "_ASM" {
        asm_error!(
            info,
            "No definition for label '{name}' found {}{}",
            hint!(
                "For some features, like dereferencing with the * operator, the assembler requires an _ASM label"
            ),
            hint!(
                "Add the definition '_ASM -> 0' somewhere in your code, or import the standard lib"
            )
        );
    }
    asm_error!(info, "No definition for label '{name}' found");
}

/*
    _ASM    _ASM    &1
    *ID*ptr *ID*ptr &1
    ptr     _ASM    &1
    _ASM    *ID*ptr &1
    a -= (*ID*ptr -> 0)
*/
fn make_deref_instructions(
    info: &Info,
    origin_info: &Vec<(i32, Info)>,
    label_with_id: &String,
    label_without_id: &String,
) -> Vec<Token> {
    let tokens_variants = vec![
        TokenVariant::Linebreak,
        TokenVariant::Label {
            name: "_ASM".to_string(),
        },
        TokenVariant::Label {
            name: "_ASM".to_string(),
        },
        TokenVariant::Relative { offset: 1 },
        TokenVariant::Linebreak,
        TokenVariant::Label {
            name: label_with_id.clone(),
        },
        TokenVariant::Label {
            name: label_with_id.clone(),
        },
        TokenVariant::Relative { offset: 1 },
        TokenVariant::Linebreak,
        TokenVariant::Label {
            name: label_without_id.to_string(),
        },
        TokenVariant::Label {
            name: "_ASM".to_string(),
        },
        TokenVariant::Relative { offset: 1 },
        TokenVariant::Linebreak,
        TokenVariant::Label {
            name: "_ASM".to_string(),
        },
        TokenVariant::Label {
            name: label_with_id.clone(),
        },
        TokenVariant::Relative { offset: 1 },
        TokenVariant::Linebreak,
    ];

    let mut deref: Vec<Token> = Vec::with_capacity(tokens_variants.len());
    for i in tokens_variants {
        deref.push(Token {
            info: info.clone(),
            origin_info: origin_info.clone(),
            variant: i,
        })
    }

    return deref;
}

pub fn expand_derefs(tokens: &Vec<Token>) -> Vec<Token> {
    const INSERTED_INSTRUCTIONS_SIZE: usize = 17;

    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    let mut last_linebreak_idx = 0;
    let mut id = 0;
    let mut address = 0;
    while i < tokens.len() {
        match tokens[i].variant {
            TokenVariant::Asterisk => {
                if i + 1 < tokens.len()
                    && let TokenVariant::Label { name } = &tokens[i + 1].variant
                {
                    let info = &tokens[i].info;
                    let origin_info = &tokens[i].origin_info;
                    let in_instruction_label = format!("*{id}*{name}");

                    let deref =
                        make_deref_instructions(info, origin_info, &in_instruction_label, name);
                    address += 12;
                    new_tokens.splice(
                        last_linebreak_idx..last_linebreak_idx,
                        deref.iter().cloned(),
                    );
                    new_tokens.push(Token {
                        info: info.clone(),
                        origin_info: tokens[i].origin_info.clone(),
                        variant: TokenVariant::BracedLabelDefinition {
                            name: in_instruction_label,
                            data: tokens::IntOrString::Int(0),
                        },
                    });
                    last_linebreak_idx += INSERTED_INSTRUCTIONS_SIZE;
                    id += 1;
                    i += 2;
                    address += 1;
                }
            }
            TokenVariant::Linebreak => {
                last_linebreak_idx = i + id * (INSERTED_INSTRUCTIONS_SIZE - 1);
                address += tokens[i].size();
                new_tokens.push(tokens[i].clone());
                i += 1;
            }
            _ => {
                address += tokens[i].size();

                new_tokens.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    new_tokens
}
