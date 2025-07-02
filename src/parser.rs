use crate::tokens::{self, Token};
use core::panic;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::os::windows::thread;
use std::rc::{Rc, Weak};
use std::thread::current;
use std::vec;
use std::{fs::File, io::Write};
use simple_logger::SimpleLogger;


#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Instruction { a: Token, b: Token, c: Token },
    ScopeControl { x: Token },
    PointerDefinition { label: Token, value: Token },
    Literal { x: Token },
}

impl Statement {
    fn size(&self) -> i32 {
        match self {
            Statement::Instruction { .. } => 3,
            Statement::PointerDefinition { .. } => 1,
            Statement::ScopeControl { .. } => 0,
            Statement::Literal { .. } => 1,
        }
    }
}

#[derive(Debug)]
struct Macro {
    args: Vec<Token>,
    body: Vec<Token>,
}

// iterator

fn read_macros(tokens: Vec<Token>) -> (Vec<Token>, HashMap<String, Macro>) {
    let mut new_tokens: Vec<Token> = Vec::new();
    let mut macros: HashMap<String, Macro> = HashMap::new();

    enum Mode {
        NORMAL,
        ARGS,
        BODY,
    }
    let mut mode: Mode = Mode::NORMAL;

    let mut macro_name: String = String::new();
    let mut macro_body: Vec<Token> = Vec::new();
    let mut macro_args: Vec<Token> = Vec::new();

    for token in tokens {
        match mode {
            Mode::NORMAL => match token {
                Token::MacroStart { name } => {
                    macro_name = name.clone();
                    mode = Mode::ARGS;
                    continue;
                }
                _ => {
                    new_tokens.push(token);
                    continue;
                }
            },
            Mode::ARGS => match &token {
                Token::Label { name: name } => {
                    macro_args.push(token);
                    continue;
                }
                Token::StatementEnd => {
                    mode = Mode::BODY;
                    continue;
                }
                _ => {
                    panic!(
                        "Only labels may be used for the macro header of '{}'.",
                        macro_name
                    )
                }
            },
            Mode::BODY => match token {
                Token::MacroEnd => {
                    let new_macro = Macro {
                        args: macro_args,
                        body: macro_body,
                    };
                    macros.insert(macro_name, new_macro);
                    macro_body = Vec::new();
                    macro_args = Vec::new();
                    macro_name = String::new();
                    mode = Mode::NORMAL;
                    continue;
                }
                _ => {
                    macro_body.push(token.clone());
                    continue;
                }
            },
        }
    }
    return (new_tokens, macros);
}

fn generate_macro_body(current_macro: &Macro, label_map: &HashMap<String, String>) -> Vec<Token> {
    let mut body: Vec<Token> = current_macro.body.clone();
    println!("{:?}", label_map);
    for body_token in &mut body {
        if let Token::Label { name } = body_token {
            let new_name = label_map.get(name);
            match new_name {
                Some(name) => *body_token = Token::Label { name: name.clone() },
                None => {
                    panic!()
                }
            }
        }
    }
    return body;
}

fn insert_macros(tokens: Vec<Token>, macros: &HashMap<String, Macro>) -> (bool, Vec<Token>) {
    let mut new_tokens: Vec<Token> = Vec::new();
    enum Mode {
        NORMAL,
        ARGS,
    }
    let mut has_inserted_macro = false;
    let mut mode = Mode::NORMAL;
    let mut current_macro: Option<&Macro> = None;
    let mut label_map: HashMap<String, String> = HashMap::new();

    for token in tokens {
        match mode {
            Mode::NORMAL => match token {
                Token::MacroCall { name } => {
                    let mac = macros.get(&name);
                    match mac {
                        None => {
                            panic!("No definition found for the macro '{}'", name);
                        }
                        Some(x) => {
                            current_macro = Some(x);
                            mode = Mode::ARGS;
                        }
                    }
                    continue;
                }
                _ => {
                    new_tokens.push(token.clone());
                }
            },
            Mode::ARGS => {
                let current_macro = current_macro.expect("Unreachable");
                match token {
                    Token::Label {
                        name: to_replace_name,
                    } => {
                        let label_to_replace_with = &current_macro.args[label_map.len()];
                        match label_to_replace_with {
                            Token::Label { name } => {
                                label_map.insert(name.clone(), to_replace_name.clone());
                            }
                            _ => {
                                panic!("Unreachable");
                            }
                        }
                        continue;
                    }
                    Token::StatementEnd => {
                        let mut body = generate_macro_body(current_macro, &label_map);
                        new_tokens.append(&mut body);
                        has_inserted_macro = true;
                        mode = Mode::NORMAL;
                        continue;
                    }
                    _ => {
                        panic!("Only labels may follow a macro call.")
                    }
                }
            }
        }
    }

    return (has_inserted_macro, new_tokens);
}

fn remove_repeating_statement_ends(tokens: &mut Vec<Token>) -> Vec<Token> {
    let mut result: Vec<Token> = Vec::new();

    let mut prev: Option<Token> = None;
    for token in tokens.iter() {
        match prev {
            Some(Token::StatementEnd) => {
                if *token == Token::StatementEnd {
                    continue;
                }
            }
            _ => {}
        }
        result.push(token.clone());
        prev = Some(token.clone());
    }
    return result;
}

fn separate_statements(tokens: &Vec<Token>) -> Vec<Statement> {
    let mut statements: Vec<Statement> = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        if tokens[idx] == Token::StatementEnd {
            idx += 1;
            continue;
        }

        if tokens[idx] == Token::Scope || tokens[idx] == Token::Unscope {
            statements.push(Statement::ScopeControl {
                x: tokens[idx].clone(),
            });
            idx += 1;
            continue;
        }

        if idx + 2 < tokens.len() && tokens[idx + 1] == Token::Pointer {
            statements.push(Statement::PointerDefinition {
                label: tokens[idx].clone(),
                value: tokens[idx + 2].clone(),
            });

            idx += 3;
            continue;
        }

        if idx + 2 < tokens.len() && tokens[idx + 2] == Token::StatementEnd {
            statements.push(Statement::Instruction {
                a: tokens[idx].clone(),
                b: tokens[idx + 1].clone(),
                c: Token::Relative { offset: 3 },
            });
            idx += 3;
            continue;
        }

        if idx + 3 < tokens.len() && tokens[idx + 3] == Token::StatementEnd {
            statements.push(Statement::Instruction {
                a: tokens[idx].clone(),
                b: tokens[idx + 1].clone(),
                c: tokens[idx + 2].clone(),
            });
            idx += 4;
            continue;
        }

        if tokens[idx] != Token::StatementEnd {
            statements.push(Statement::Literal {
                x: tokens[idx].clone(),
            });
        }
        idx += 1;
    }
    return statements;
}

fn hex_to_dec(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        match token {
            Token::HexLiteral { value } => {
                *token = Token::DecLiteral {
                    value: i32::from_str_radix(value, 16).expect("Should be hex."),
                };
            }
            _ => continue,
        }
    }
}

fn assign_addresses_to_labels(statements: &Vec<Statement>) -> Vec<HashMap<String, i32>> {
    let mut scopes: Vec<HashMap<String, i32>> = vec![HashMap::new()];
    let mut address: i32 = 0;
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;
    for statement in statements {
        match statement {
            Statement::ScopeControl { x } => match x {
                Token::Scope => {
                    scopes.push(HashMap::new());
                    let current_scope_idx = seen_scopes_count + 1;
                    current_scope_indexes.push(current_scope_idx);
                    println!("SCOPE {:?}", current_scope_indexes);
                    seen_scopes_count += 1;
                }
                Token::Unscope => {
                    current_scope_indexes.pop();
                    println!("UNSCOPE {:?}", current_scope_indexes);
                }

                _ => panic!("Non scope token in scope control."),
            },

            Statement::PointerDefinition { label, value } => match label {
                Token::Label { name } => {
                    scopes[current_scope_indexes[current_scope_indexes.len() - 1]]
                        .insert(name.clone(), address);
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

fn resolve_labels(statements: &mut Vec<Statement>, scoped_label_table: &Vec<HashMap<String, i32>>) {
    let mut current_scope_indexes: Vec<usize> = vec![0];
    let mut seen_scopes_count: usize = 0;

    for statement in statements {
        match statement {
            Statement::ScopeControl { x } => match x {
                Token::Scope => {
                    let current_scope_idx = seen_scopes_count + 1;
                    current_scope_indexes.push(current_scope_idx);
                    seen_scopes_count += 1;
                }
                Token::Unscope => {
                    current_scope_indexes.pop();
                }
                _ => panic!("Non scope token in scope control."),
            },
            _ => {}
        }
        if let Statement::Instruction { a, b, c } = statement {
            if let Token::Label { name } = a {
                *a = Token::DecLiteral {
                    value: find_label(name, scoped_label_table, &current_scope_indexes),
                }
            }
            if let Token::Label { name } = b {
                *b = Token::DecLiteral {
                    value: find_label(name, scoped_label_table, &current_scope_indexes),
                }
            }
            if let Token::Label { name } = c {
                *c = Token::DecLiteral {
                    value: find_label(name, scoped_label_table, &current_scope_indexes),
                }
            }
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
    panic!("Label '{}' is undefined.", name)
}

fn resolve_relatives(statements: &mut Vec<Statement>) {
    let mut address: i32 = 0;

    for statement in statements {
        if let Statement::Instruction { a, b, c } = statement {
            if let Token::Relative { offset } = a {
                *a = Token::DecLiteral {
                    value: address + *offset,
                }
            }
            if let Token::Relative { offset } = b {
                *b = Token::DecLiteral {
                    value: address + *offset,
                }
            }
            if let Token::Relative { offset } = c {
                *c = Token::DecLiteral {
                    value: address + *offset,
                }
            }
        }
        address += statement.size();
    }
}

fn loop_insert_macros(tokens: Vec<Token>, macros: &HashMap<String, Macro>) -> Vec<Token> {
    let mut has_inserted = false;
    let mut t = tokens;

    loop {
        (has_inserted, t) = insert_macros(t, &macros);
        if !has_inserted {
            return t;
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {

    println!("{:?}", tokens);

    let (mut tokens, macros) = read_macros(tokens);
    println!("{:?}", macros);

    tokens = loop_insert_macros(tokens, &macros);

    for token in &tokens {
        println!("{:?}", token);
    }

    //return;

    hex_to_dec(&mut tokens);

    let mut statements = separate_statements(&tokens);
    let scoped_label_table = assign_addresses_to_labels(&statements);
    for statement in &statements {
        println!("{:?}", statement);
    }

    println!();

    //   let label_table: HashMap<String, i32> = assign_addresses_to_labels(&statements);
    resolve_labels(&mut statements, &scoped_label_table);
    for statement in &statements {
        println!("{:?}", statement);
    }

    resolve_relatives(&mut statements);
    return statements;
}
