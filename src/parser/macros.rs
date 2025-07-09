use std::collections::HashMap;
use std::thread::scope;
use crate::feedback::*;
use crate::hint;
use crate::println_debug;
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
        BODY_BOUNDED_BY_SCOPES,
    }
    let mut mode: Mode = Mode::NORMAL;
    let mut scope_tracker = 0;


    let mut macro_name: String = String::new();
    let mut macro_body: Vec<Token> = Vec::new();
    let mut macro_args: Vec<String> = Vec::new();

    for token in tokens {
        match mode {
            Mode::NORMAL => match token {
                Token::MacroDeclaration { info, name } => {
                    macro_name = name.clone();
                    if macros.contains_key(&macro_name) {
                        asm_warn!(&info, "A macro with this name has already been defined");
                    }
                    mode = Mode::ARGS;
                    continue;
                }
                Token::MacroBodyStart { info } | Token::MacroBodyEnd { info } => {
                    asm_error!(&info, "Unexpected token");
                }
                _ => {
                    new_tokens.push(token);
                    continue;
                }
            },
            Mode::ARGS => match &token {
                Token::Linebreak {..} => {
                    continue;
                }
                Token::Label { info, name: name } => {
                    macro_args.push(name.clone());
                    if !name.ends_with('?') {
                        asm_warn!(info, "Notate macro arguments with a trailing question mark {}", hint!("'{name}' -> '{name}?'"));
                    }
                    continue;
                }
                Token::MacroBodyStart {info}=> {
                    mode = Mode::BODY;
                    continue;
                }
                Token::Scope { info } => {
                    mode = Mode::BODY_BOUNDED_BY_SCOPES;
                    macro_body.push(token.clone());
                    scope_tracker += 1;
                    continue;
                }

                tok => {
                    asm_error!(tok.get_info(), "Only labels may be used as arguments for '{macro_name}'");
                }
            },
            Mode::BODY => match &token {
                Token::LabelArrow { info, offset } => {
                    macro_body.push(token.clone());
                    asm_warn!(info, "Label definitions in non-scoped macros may cause undesired behaviour {}", hint!("Use '{{' and '}}' instead of '[' and ']'"));
                }

                // Token::macrostart error
                Token::MacroBodyEnd {info} => {
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
            Mode::BODY_BOUNDED_BY_SCOPES => match &token {
                Token::Scope {info} => {

                    macro_body.push(token.clone());
                    scope_tracker += 1;
                    continue;
                }


                Token::Unscope {info} => {
                    macro_body.push(token.clone());
                    scope_tracker -= 1;
                    if scope_tracker != 0 {

                        continue;
                    }

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
            }
        }
    }
    return (new_tokens, macros);
}

fn generate_macro_body(current_macro: &Macro, label_map: &HashMap<String, Token>) -> Vec<Token> {
    let mut body: Vec<Token> = current_macro.body.clone();
    println_debug!("{:?}", label_map);
    for body_token in &mut body {
        if let Token::Label { info, name } = body_token {
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
                Token::MacroCall { info, name } => {
                    let mac = macros.get(&name);
                    match mac {
                        None => {
                            asm_error!(&info, "No declaration found for the macro '{name}'.");
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
    let mut has_inserted ;
    let mut t = tokens;

    loop {
        (has_inserted, t) = insert_macros(t, &macros);
        if !has_inserted {
            return t;
        }
    }
}
