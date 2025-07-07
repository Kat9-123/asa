use crate::asm_error;
use crate::tokens::{self, Token};
use core::panic;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::os::windows::thread;
use std::path::Iter;
use std::rc::{Rc, Weak};
use std::thread::current;
use std::vec;
use std::{fs::File, io::Write};
use simple_logger::SimpleLogger;


#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Instruction { a: Token, b: Token, c: Token },
    Control { x: Token },
    LabelDefinition {label: Token},
    Literal { x: Token },
}

impl Statement {
    fn size(&self) -> i32 {
        match self {
            Statement::Instruction { .. } => 3,
            Statement::LabelDefinition { .. } => 0,
            Statement::Control { .. } => 0,
            Statement::Literal { .. } => 1,
        }
    }
}
// iterator
impl Iterator for Statement {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}



#[derive(Debug)]
struct Macro {
    args: Vec<Token>,
    body: Vec<Token>,
}


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
                Token::MacroDeclaration { name } => {
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
                Token::StatementEnd => {
                    continue;
                }
                Token::Label { name: name } => {
                    macro_args.push(token);
                    continue;
                }
                Token::MacroBodyStart => {
                    mode = Mode::BODY;
                    continue;
                }

                _ => {
                    asm_error!("Only labels may be used as arguments for '{macro_name}'.");
                }
            },
            Mode::BODY => match token {
                // Token::macrostart error
                Token::MacroBodyEnd => {
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

fn generate_macro_body(current_macro: &Macro, label_map: &HashMap<String, Token>) -> Vec<Token> {
    let mut body: Vec<Token> = current_macro.body.clone();
    println!("{:?}", label_map);
    for body_token in &mut body {
        if let Token::Label { name } = body_token {
            let new_token = label_map.get(name);
            match new_token {
                Some(t) => *body_token = t.clone(),
                None => {
                    continue;
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
    let mut label_map: HashMap<String, Token> = HashMap::new();

    for token in tokens {
        match mode {
            Mode::NORMAL => match token {
                Token::MacroCall { name } => {
                    let mac = macros.get(&name);
                    match mac {
                        None => {
                            asm_error!("No declaration found for the macro '{name}'.");
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

                let current_macro_safe = current_macro.unwrap();

                if label_map.len() >= current_macro_safe.args.len() {
                    let mut body = generate_macro_body(current_macro_safe, &label_map);
                    new_tokens.append(&mut body);
                    has_inserted_macro = true;
                    mode = Mode::NORMAL;
                    current_macro = None;
                    label_map = HashMap::new();
                    new_tokens.push(token.clone());

                    continue;
                }

                let label_to_replace = &current_macro_safe.args[label_map.len()];
                match label_to_replace {
                    Token::Label { name } => {
                        label_map.insert(name.clone(), token.clone());
                    }
                    _ => {
                        panic!("Unreachable");
                    }
                }

                continue;
            }
        }
    }

    return (has_inserted_macro, new_tokens);
}

fn separate_statements(tokens: &Vec<Token>) -> Vec<Statement> {
    let mut statements: Vec<Statement> = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        if tokens[idx] == Token::StatementEnd {
            idx += 1;
            continue;
        }
        match tokens[idx] {
            Token::Scope | Token::Unscope | Token::Namespace { .. } => {
                statements.push(Statement::Control {
                    x: tokens[idx].clone(),
                });
                idx += 1;
                continue;
            }

            _=> {}
        }


        if idx + 2 < tokens.len() && tokens[idx + 1] == Token::LabelArrow {

            statements.push(Statement::LabelDefinition {
                label: tokens[idx].clone(),
            });

            idx += 2;
            continue;
        }

        if tokens[idx + 1] == Token::Subleq {
            if idx + 3 < tokens.len() && tokens[idx + 3] == Token::StatementEnd {
                statements.push(Statement::Instruction {
                    // Subleq has a and b flipped
                    a: tokens[idx + 2].clone(),
                    b: tokens[idx].clone(),
                    c: Token::Relative { offset: 1 },
                });
                idx += 4;
                continue;
            }
            if idx + 4 < tokens.len() && tokens[idx + 4] == Token::StatementEnd {
                statements.push(Statement::Instruction {
                    // Subleq has a and b flipped
                    a: tokens[idx + 2].clone(),
                    b: tokens[idx].clone(),
                    c: tokens[idx + 3].clone(),
                });
                idx += 5;
                continue;
            }

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



fn char_and_hex_to_dec(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        match token {
            Token::HexLiteral { value } => {
                *token = Token::DecLiteral {
                    value: i32::from_str_radix(value, 16).expect("Should be hex."),
                };
            }
            Token::CharLiteral { value } => {
                *token = Token::DecLiteral {
                    value: *value as i32
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
   // let mut namespace: String = String::from("");

    for statement in statements {
        match statement {
            Statement::Control { x } => match x {
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
                Token::Namespace { name } => {
                    println!("set namespace to {name}");

                   // namespace = name.clone();
                }

                _ => panic!("Non control in control statement."),
            },

            Statement::LabelDefinition { label} => match label {
                Token::Label { name } => {
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
            Statement::Control { x } => match x {
                Token::Scope => {
                    let current_scope_idx = seen_scopes_count + 1;
                    current_scope_indexes.push(current_scope_idx);
                    seen_scopes_count += 1;
                }
                Token::Unscope => {
                    current_scope_indexes.pop();
                }
                Token::Namespace {..} => {  },
                _ => { asm_error!("Non control in control statement."); }
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
    asm_error!("Label '{}' is undefined.", name);
}

fn resolve_relatives(statements: &mut Vec<Statement>) {
    let mut address: i32 = 0;

    for statement in statements {
        match statement {
            Statement::Instruction { a, b, c } => {
                if let Token::Relative { offset } = a {
                    *a = Token::DecLiteral {
                        value: address + *offset,
                    }
                }
                if let Token::Relative { offset } = b {
                    *b = Token::DecLiteral {
                        value: address + 1 + *offset,
                    }
                }
                if let Token::Relative { offset } = c {
                    *c = Token::DecLiteral {
                        value: address + 2 + *offset,
                    }
                }
            },
            Statement::Literal { x } => {
                if let Token::Relative { offset } = x {
                    *x = Token::DecLiteral {
                        value: address + *offset,
                    }
                }
            },

            _ => {}

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


fn expand_strings(tokens: Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();
    for token in tokens {
        match token {
            Token::StrLiteral { value } => {
                for c in value.chars() {
                    new_tokens.push(Token::CharLiteral { value: c });
                }
            }
            _ => new_tokens.push(token)
        }
    }


    return new_tokens;
}

pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {

    

    let (mut tokens, macros) = read_macros(tokens);

    log::debug!("Found macros:");
    for i in &macros {
        println!("{i:?}");

    }
    println!();


    tokens = loop_insert_macros(tokens, &macros);

    log::debug!("Inserted macros:");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();


    let mut tokens = expand_strings(tokens);
    char_and_hex_to_dec(&mut tokens);
    
    log::debug!("Converted literals:");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();




    let mut statements = separate_statements(&tokens);

    log::debug!("Statements");
    for statement in &statements {
        println!("{:?}", statement);
    }
    println!();


    let scoped_label_table = assign_addresses_to_labels(&statements);

    log::debug!("Label Table");
    println!("{:?}", scoped_label_table);
    println!();

    log::debug!("Label Table");

    //   let label_table: HashMap<String, i32> = assign_addresses_to_labels(&statements);
    resolve_labels(&mut statements, &scoped_label_table);
    for statement in &statements {
        println!("{:?}", statement);
    }
    println!();



    resolve_relatives(&mut statements);
    for statement in &statements {
        println!("{:?}", statement);
    }
    return statements;
}
