use std::collections::HashMap;

use crate::asm_details;
use crate::asm_error;
use crate::asm_error_no_terminate;
use crate::asm_hint;
use crate::asm_warn;
use crate::terminate;
use crate::tokens;
use crate::tokens::*;
use colored::Colorize;

/// Labels may be defined inside of instructions using the following syntax:
/// (label -> 0). This routine converts these definitions into single tokens
pub fn grab_braced_label_definitions(tokens: Vec<Token>) -> Vec<Token> {
    let mut updated_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;

    while i < tokens.len() {
        if let TokenVariant::BraceOpen = &tokens[i].variant
            && let TokenVariant::LabelArrow { .. } = &tokens[i + 2].variant
        {
            let name = match &tokens[i + 1].variant {
                TokenVariant::Label { name } => name,
                _ => asm_error!(&tokens[i + 1].info, "Unexpected token, expected a LABEL"),
            };
            let data: IntOrString = match &tokens[i + 3].variant {
                TokenVariant::DecLiteral { value } => IntOrString::Int(*value),
                TokenVariant::Label { name } => IntOrString::Str(name.clone()),
                _ => asm_error!(
                    &tokens[i + 3].info,
                    "Unexpected token, expected a LABEL or LITERAL"
                ),
            };

            updated_tokens.push(Token::with_info(
                // take the info of the value pointed at
                TokenVariant::BracedLabelDefinition {
                    name: name.clone(),
                    data,
                },
                &tokens[i + 2],
            ));
            i += 5;
            continue;
        }
        updated_tokens.push(tokens[i].clone());
        i += 1;
    }

    updated_tokens
}

/// Find every label definition, and store the address that it should point to
/// Returns a vector with a hashmap for each scope, containing key value pairs of the name
/// of the label and its value and info
pub fn assign_addresses_to_labels(tokens: &[Token]) -> Vec<HashMap<String, (usize, Info)>> {
    fn new_label(
        current_scope: &mut HashMap<String, (usize, Info)>,
        name: &String,
        address: usize,
        info: &Info,
    ) {
        if let Some(x) = current_scope.get(name) {
            asm_warn!(
                info,
                "The label called '{name}' has already been defined in this scope"
            );
            asm_details!(&x.1, "Here");
        }

        current_scope.insert(name.clone(), (address, info.clone()));
    }

    let mut scopes: Vec<HashMap<String, (usize, Info)>> = vec![HashMap::new()];
    let mut address: usize = 0;
    // Stack maintaining the indices for the scopes. Top is the current one. They
    // index the scope vec
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for token in tokens {
        match &token.variant {
            TokenVariant::Scope => {
                scopes.push(HashMap::new());
                let current_scope_idx = seen_scopes_count + 1;
                current_scope_indexes.push(current_scope_idx);
                seen_scopes_count += 1;
            }
            TokenVariant::Unscope => {
                current_scope_indexes.pop();
            }

            TokenVariant::BracedLabelDefinition { name, .. } => {
                let current_scope =
                    &mut scopes[current_scope_indexes[current_scope_indexes.len() - 1]];
                new_label(current_scope, name, address, &token.info);
            }

            TokenVariant::LabelDefinition { name, offset } => {
                let current_scope =
                    &mut scopes[current_scope_indexes[current_scope_indexes.len() - 1]];

                new_label(
                    current_scope,
                    name,
                    address + (*offset) as usize,
                    &token.info,
                );
            }

            _ => {}
        }
        address += token.size();
    }

    scopes
}

/// All labels get resolved, i.e. converted into the address they label.
/// This routine also resolves relatives.
pub fn resolve_labels_and_relatives(
    tokens: &mut [Token],
    scoped_label_table: &[HashMap<String, (usize, Info)>],
) {
    fn find_label(
        name: &String,
        scoped_label_table: &[HashMap<String, (usize, Info)>],
        current_scope_indexes: &[usize],
        #[allow(unused_variables)] info: &Info, // Maybe I'm missing something, but this variable is most definitely used.
    ) -> (usize, Info) {
        for scope in current_scope_indexes.iter().rev() {
            if let Some(x) = scoped_label_table[*scope].get(name) {
                return x.clone();
            }
        }
        if name == "_ASM" {
            asm_error_no_terminate!(info, "No definition for label '{name}' found",);
            asm_hint!(
                "For some features, like dereferencing with the * operator, the assembler requires an _ASM label"
            );
            asm_hint!(
                "Add the definition '_ASM -> 0' somewhere in your code, or import the standard lib"
            );
            terminate!();
        }
        asm_error!(info, "No definition for label '{name}' found");
    }

    let mut address: usize = 0;
    // Scope index stack
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for token in tokens.iter_mut() {
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
                let (val, _) = find_label(
                    name,
                    scoped_label_table,
                    &current_scope_indexes,
                    &token.info,
                );
                token.variant = TokenVariant::DecLiteral { value: val as i32 };
            }
            TokenVariant::BracedLabelDefinition { name, data } => {
                let value = match data {
                    IntOrString::Int(val) => *val,
                    IntOrString::Str(..) => {
                        let (val, _) = find_label(
                            name,
                            scoped_label_table,
                            &current_scope_indexes,
                            &token.info,
                        );
                        val as i32
                    }
                };

                token.variant = TokenVariant::DecLiteral { value };
            }
            // a &2 => a 3
            TokenVariant::Relative { offset } => {
                token.variant = TokenVariant::DecLiteral {
                    value: address as i32 + offset,
                };
            }

            _ => {}
        }
        address += token.size();
    }
}

/*
    _ASM    _ASM    &1
    *ID*ptr *ID*ptr &1
    ptr     _ASM    &1
    _ASM    *ID*ptr &1
*/
fn make_deref_instructions(
    info: &Info,
    origin_info: &[Info],
    label_with_id: &str,
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
            name: label_with_id.to_owned(),
        },
        TokenVariant::Label {
            name: label_with_id.to_owned(),
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
            name: label_with_id.to_owned(),
        },
        TokenVariant::Relative { offset: 1 },
        TokenVariant::Linebreak,
    ];

    let mut deref: Vec<Token> = Vec::with_capacity(tokens_variants.len());
    for i in tokens_variants {
        deref.push(Token {
            info: info.clone(),
            origin_info: origin_info.to_owned(),
            variant: i,
        })
    }

    deref
}

/// Messy
pub fn expand_derefs(tokens: &[Token]) -> Vec<Token> {
    const INSERTED_INSTRUCTIONS_SIZE: usize = 17;

    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    let mut last_linebreak_idx = 0;
    let mut id = 0;
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
                }
            }
            TokenVariant::Linebreak => {
                last_linebreak_idx = i + id * (INSERTED_INSTRUCTIONS_SIZE - 1);
                new_tokens.push(tokens[i].clone());
                i += 1;
            }
            _ => {
                new_tokens.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    new_tokens
}

#[cfg(test)]
mod tests {

    use crate::tokens::tokens_from_token_variant_vec;

    use super::*;

    #[test]
    fn braced_labels() {
        let input: Vec<Token> = tokens_from_token_variant_vec(vec![
            (0, TokenVariant::BraceOpen),
            (
                1,
                TokenVariant::Label {
                    name: "label1".to_string(),
                },
            ),
            (
                2,
                TokenVariant::LabelArrow {
                    offset: LabelOffset::Int(0),
                },
            ),
            (3, TokenVariant::DecLiteral { value: 1234 }),
            (4, TokenVariant::BraceClose),
            (5, TokenVariant::BraceOpen),
            (
                6,
                TokenVariant::Label {
                    name: "label2".to_string(),
                },
            ),
            (
                7,
                TokenVariant::LabelArrow {
                    offset: LabelOffset::Int(0),
                },
            ),
            (
                8,
                TokenVariant::Label {
                    name: "TEXT".to_string(),
                },
            ),
            (9, TokenVariant::BraceClose),
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                2,
                TokenVariant::BracedLabelDefinition {
                    name: "label1".to_string(),
                    data: IntOrString::Int(1234),
                },
            ),
            (
                7,
                TokenVariant::BracedLabelDefinition {
                    name: "label2".to_string(),
                    data: IntOrString::Str("TEXT".to_string()),
                },
            ),
        ]);
        let output = grab_braced_label_definitions(input);
        assert_eq!(output, expected);
    }
}
