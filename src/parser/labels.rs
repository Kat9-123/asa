use std::collections::HashMap;

use crate::feedback::*;
use crate::println_debug;
use crate::tokens::*;


pub fn grab_braced_label_definitions(tokens: Vec<Token>) -> Vec<Token> {
    let mut updated_tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if let Token::BraceOpen { info } = &tokens[i] {
            let name = match &tokens[i + 1] {
                Token::Label { info, name } => name,
                _ => todo!(),
            };
            let data: IntOrString = match &tokens[i + 3] {
                Token::DecLiteral { info, value } => IntOrString::Int(*value),  
                Token::Label { info, name  } => IntOrString::Str(name.clone()),
                _ => todo!()
            };

            updated_tokens.push(Token::BracedLabelDefinition {
                info: info.clone(),
                name: name.clone(),
                data: data,
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
   // let mut namespace: String = String::from("");

    for token in tokens {

        match token {
            Token::Scope {info} => {
                scopes.push(HashMap::new());
                let current_scope_idx = seen_scopes_count + 1;
                current_scope_indexes.push(current_scope_idx);
                println_debug!("SCOPE {:?}", current_scope_indexes);
                seen_scopes_count += 1;
            }
            Token::Unscope {info }=> {
                current_scope_indexes.pop();
                println_debug!("UNSCOPE {:?}", current_scope_indexes);
            }
            Token::Namespace {info, name } => {
                println_debug!("set namespace to {name}");

                // namespace = name.clone();
            }
            Token::BracedLabelDefinition { info, name, data } => {
                if scopes[current_scope_indexes[current_scope_indexes.len() - 1]].contains_key(name) {
                    asm_warn!(info, "The label called '{name}' has already been defined in this scope");
                }
                scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                        .insert(name.clone(), (address, info.clone()));
            }


            Token::LabelDefinition {info, name, offset} => {
                    /*
                    let mut name_with_scope: String;
                    if &namespace != "THIS" {
                        name_with_scope = namespace.clone();
                        name_with_scope.push_str("::");
                        name_with_scope.push_str(&name);
                    } else {
                        name_with_scope = name.to_string();
                    } 

                    println_debug!("{name_with_scope}"); */
                    if scopes[current_scope_indexes[current_scope_indexes.len() - 1]].contains_key(name) {
                        asm_warn!(info, "The label called '{name}' has already been defined in this scope");
                    }

                    scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                        .insert(name.clone(), (address + *offset, info.clone()));

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
        match token {

            Token::Scope {info} => {
                let current_scope_idx = seen_scopes_count + 1;
                current_scope_indexes.push(current_scope_idx);
                seen_scopes_count += 1;
            }
            Token::Unscope {info} => {
                current_scope_indexes.pop();
            }
            Token::Label { info: _info, name } => {
                let (val, inf) = find_label(&name, scoped_label_table, &current_scope_indexes, _info);
                updated_tokens.push(Token::DecLiteral {
                        info: _info.clone(),
                        value: val,
                });
            }
            Token::BracedLabelDefinition { info: info, name, data } => {
                match data {
                    IntOrString::Int(x) => updated_tokens.push(Token::DecLiteral { info: info.clone(), value: *x }),
                    IntOrString::Str(string) => {

                        let (val, inf) = find_label(&name, scoped_label_table, &current_scope_indexes, info);
                        updated_tokens.push(Token::DecLiteral {
                                info: info.clone(),
                                value: val,
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
