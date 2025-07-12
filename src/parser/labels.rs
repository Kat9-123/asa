use std::collections::HashMap;

use crate::asm_details;
use crate::feedback::*;
use crate::println_debug;
use crate::tokens::*;


pub fn grab_braced_label_definitions(tokens: Vec<Token>) -> Vec<Token> {
    let mut updated_tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if let TokenVariant::BraceOpen  = &tokens[i].variant {
            let name = match &tokens[i + 1].variant {
                TokenVariant::Label { name } => name,
                _ => todo!(),
            };
            let data: IntOrString = match &tokens[i + 3].variant {
                TokenVariant::DecLiteral { value } => IntOrString::Int(*value),
                TokenVariant::Label {name  } => IntOrString::Str(name.clone()),
                _ => todo!()
            };

            updated_tokens.push(Token {
                info: tokens[i].info.clone(),
                variant: TokenVariant::BracedLabelDefinition {
                    name: name.clone(),
                    data: data,
                }
            }
            );
            i += 5;
            continue;
        }
        updated_tokens.push(tokens[i].clone());
        i += 1;
    }

    return updated_tokens;
}

pub fn assign_addresses_to_labels(tokens: &Vec<Token>) -> Vec<HashMap<String, (i32, Info)>> {
    let mut scopes: Vec<HashMap<String, (i32, Info)>> = vec![HashMap::new()];
    let mut address: i32 = 0;
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for token in tokens {

        match &token.variant {
            TokenVariant::Scope  => {
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
                match scopes[current_scope_indexes[current_scope_indexes.len() - 1]].get(name) {
                    Some(x) => {
                        asm_warn!(&token.info, "The label called '{name}' has already been defined in this scope");
                        asm_details!(&x.1, "Here");
                    }
                    None => {}
                }

                scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                        .insert(name.clone(), (address, token.info.clone()));
            }


            TokenVariant::LabelDefinition { name, offset} => {
                    match scopes[current_scope_indexes[current_scope_indexes.len() - 1]].get(name) {
                        Some(x) => {
                            asm_warn!(&token.info, "The label called '{name}' has already been defined in this scope");
                            asm_details!(&x.1, "Here");
                        }
                        None => {}
                    }

                    scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                        .insert(name.clone(), (address + offset, token.info.clone()));

            },

            _ => {}
        }
        address += token.size();
    }

    println_debug!("{:?}", scopes);
    return scopes;
}

pub fn resolve_labels(tokens: &Vec<Token>, scoped_label_table: &Vec<HashMap<String, (i32, Info)>>) -> Vec<Token> {
    let mut updated_tokens: Vec<Token> = Vec::new();

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
                let (val, inf) = find_label(&name, scoped_label_table, &current_scope_indexes, &token.info);
                updated_tokens.push(Token {
                        info: token.info.clone(),
                        variant: TokenVariant::DecLiteral {
                            value: val,
                        }
                });
            }
            TokenVariant::BracedLabelDefinition {name, data } => {
                match data {
                    IntOrString::Int(x) => updated_tokens.push(Token {info: token.info.clone(), variant: TokenVariant::DecLiteral { value: *x } }),
                    IntOrString::Str(string) => {

                        let (val, inf) = find_label(&name, scoped_label_table, &current_scope_indexes, &token.info);
                        updated_tokens.push(Token {
                            info: token.info.clone(),

                            variant: TokenVariant::DecLiteral {
                                value: val,
                            }
                        });
                    }
                }
            }
            _ => {updated_tokens.push(token.clone())}
        }

    }
    return updated_tokens;
}

fn find_label(
    name: &String,
    scoped_label_table: &Vec<HashMap<String, (i32, Info)>>,
    current_scope_indexes: &Vec<usize>,
    info: &Info
) -> (i32, Info) {

    for scope in current_scope_indexes.iter().rev() {
        match scoped_label_table[*scope].get(name) {
            Some(x) => return x.clone(),
            None => {}
        }
    };
    asm_error!(info, "No definition for label '{name}' found");
}
