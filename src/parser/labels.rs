use std::collections::HashMap;

use crate::feedback::*;
use crate::tokens::*;
use crate::parser::statements::Statement;




pub fn assign_addresses_to_labels(statements: &Vec<Statement>) -> Vec<HashMap<String, i32>> {
    let mut scopes: Vec<HashMap<String, i32>> = vec![HashMap::new()];
    let mut address: i32 = 0;
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;
   // let mut namespace: String = String::from("");

    for statement in statements {
        match statement {
            Statement::Control { x } => match x {
                Token::Scope {info} => {
                    scopes.push(HashMap::new());
                    let current_scope_idx = seen_scopes_count + 1;
                    current_scope_indexes.push(current_scope_idx);
                    println!("SCOPE {:?}", current_scope_indexes);
                    seen_scopes_count += 1;
                }
                Token::Unscope {info }=> {
                    current_scope_indexes.pop();
                    println!("UNSCOPE {:?}", current_scope_indexes);
                }
                Token::Namespace {info, name } => {
                    println!("set namespace to {name}");

                   // namespace = name.clone();
                }

                _ => panic!("Non control in control statement."),
            },

            Statement::LabelDefinition { label, offset} => match label {
                Token::Label {info, name } => {
                    /*
                    let mut name_with_scope: String;
                    if &namespace != "THIS" {
                        name_with_scope = namespace.clone();
                        name_with_scope.push_str("::");
                        name_with_scope.push_str(&name);
                    } else {
                        name_with_scope = name.to_string();
                    } 

                    println!("{name_with_scope}"); */
                    scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                        .insert(name.clone(), address + *offset);
                }
                _ => panic!("Invalid token in pointer definition."),
            },

            _ => {}
        }
        address += statement.size();
    }

    println!("{:?}", scopes);
    return scopes;
}

pub fn resolve_labels(statements: &mut Vec<Statement>, scoped_label_table: &Vec<HashMap<String, i32>>) {
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for statement in statements {
        match statement {
            Statement::Control { x } => match x {
                Token::Scope {info} => {
                    let current_scope_idx = seen_scopes_count + 1;
                    current_scope_indexes.push(current_scope_idx);
                    seen_scopes_count += 1;
                }
                Token::Unscope {info} => {
                    current_scope_indexes.pop();
                }
                Token::Namespace {..} => {  },
                _ => { asm_error!("Non control in control statement."); }
            },
            Statement::Instruction { a, b, c } => {
                if let Token::Label { info,name } = a {
                    *a = Token::DecLiteral {
                        info: info.clone(), // Probably the wrong info
                        value: find_label(name, scoped_label_table, &current_scope_indexes),
                    }
                }
                if let Token::Label {info, name } = b {
                    *b = Token::DecLiteral {
                        info: info.clone(),

                        value: find_label(name, scoped_label_table, &current_scope_indexes),
                    }
                }
                if let Token::Label {info, name } = c {
                    *c = Token::DecLiteral {
                        info: info.clone(),

                        value: find_label(name, scoped_label_table, &current_scope_indexes),
                    }
                }
            }
            Statement::Literal { x } => {
                if let Token::Label {info, name } = x {
                    *x = Token::DecLiteral {
                        info: info.clone(),
                        value: find_label(name, scoped_label_table, &current_scope_indexes),
                    }
                }
            }
            _ => {}
        }

    }
}

fn find_label(
    name: &String,
    scoped_label_table: &Vec<HashMap<String, i32>>,
    current_scope_indexes: &Vec<usize>,
) -> i32 {

    for scope in current_scope_indexes.iter().rev() {
        match scoped_label_table[*scope].get(name) {
            Some(x) => return *x,
            None => {}
        }
    }
    asm_error!("Label '{}' is undefined.", name);
}
