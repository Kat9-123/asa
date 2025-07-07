use std::collections::HashMap;
use crate::feedback::*;
use crate::tokens::*;

#[derive(Debug)]
pub struct Macro {
    args: Vec<String>,
    body: Vec<Token>,
}


pub fn read_macros(tokens: Vec<Token>) -> (Vec<Token>, HashMap<String, Macro>) {
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
    let mut macro_args: Vec<String> = Vec::new();

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
                Token::Linebreak => {
                    continue;
                }
                Token::Label { name: name } => {
                    macro_args.push(name.clone());
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

                let name_to_replace = &current_macro_safe.args[label_map.len()];
                label_map.insert(name_to_replace.clone(), token.clone());


                continue;
            }
        }
    }

    return (has_inserted_macro, new_tokens);
}

pub fn loop_insert_macros(tokens: Vec<Token>, macros: &HashMap<String, Macro>) -> Vec<Token> {
    let mut has_inserted = false;
    let mut t = tokens;

    loop {
        (has_inserted, t) = insert_macros(t, &macros);
        if !has_inserted {
            return t;
        }
    }
}
