use std::collections::HashMap;
use crate::asm_details;
use crate::asm_info;
use crate::feedback::*;
use crate::hint;
use crate::println_debug;
use crate::tokens::*;
use colored::Colorize;

#[derive(Debug)]
pub struct Macro {
    name: String,
    info: Info,
    args: Vec<String>,
    body: Vec<Token>,
    labels_defined_in_macro: Vec<String>
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
    let mut macro_info: Option<Info> = None;
    let mut in_macro_label_definitions: Vec<String> = Vec::new();

    for i in 0..tokens.len() {
        let token: &Token = &tokens[i];
        match mode {
            Mode::NORMAL => match &token.variant {
                TokenVariant::MacroDeclaration {  name } => {
                    macro_name = name.clone();
                    if let Some(x) = macros.get(&macro_name) {
                        asm_warn!(&token.info, "A macro with this name has already been defined");
                        asm_details!(&x.info, "Here");
                    }
                    macro_info = Some(token.info.clone());
                    mode = Mode::ARGS;
                    continue;
                }
                TokenVariant::MacroBodyStart | TokenVariant::MacroBodyEnd => {
                    asm_error!(&token.info, "Unexpected token");
                }
                _ => {
                    new_tokens.push(token.clone());
                    continue;
                }
            },
            Mode::ARGS => match &token.variant {
                TokenVariant::Linebreak => {
                    continue;
                }
                TokenVariant::Label { name } => {
                    macro_args.push(name.clone());
                    if !name.ends_with('?') {
                        asm_info!(&token.info, "Notate macro arguments with a trailing question mark {}", hint!("'{name}' -> '{name}?'"));
                    }
                    continue;
                }
                TokenVariant::MacroBodyStart => {
                    mode = Mode::BODY;
                    continue;
                }
                TokenVariant::Scope  => {
                    mode = Mode::BODY_BOUNDED_BY_SCOPES;
                    macro_body.push(token.clone());
                    scope_tracker += 1;
                    continue;
                }

                _ => {
                    asm_error!(&token.info, "Only labels may be used as arguments for '{macro_name}'");
                }
            },
            Mode::BODY => match &token.variant {
                TokenVariant::LabelArrow {  offset } => {
                    macro_body.push(token.clone());
                    asm_warn!(&token.info, "Label definitions in non-scoped macros are very dangerous {}", hint!("Use '{{' and '}}' instead of '[' and ']'"));
                }

                // Token::macrostart error
                TokenVariant::MacroBodyEnd => {
                    if let TokenVariant::Linebreak = macro_body[0].variant {
                        macro_body.remove(0);
                    }
                    if let TokenVariant::Linebreak = macro_body[macro_body.len() - 1].variant {
                        macro_body.remove(macro_body.len() - 1);
                    }

                    let new_macro = Macro {
                        name: macro_name.clone(),
                        info: macro_info.unwrap(),
                        args: macro_args,
                        body: macro_body,
                        labels_defined_in_macro: in_macro_label_definitions,
                    };
                    macros.insert(macro_name, new_macro);
                    macro_body = Vec::new();
                    macro_args = Vec::new();
                    macro_info = None;
                    macro_name = String::new();
                    in_macro_label_definitions = Vec::new();
                    mode = Mode::NORMAL;
                    continue;
                }
                _ => {
                    macro_body.push(token.clone());
                    continue;
                }
            },
            Mode::BODY_BOUNDED_BY_SCOPES => match &token.variant {

                // HACK
                TokenVariant::LabelArrow { offset } => {
                    macro_body.push(token.clone());
                    match &tokens[i - 1].variant {
                        TokenVariant::Label { name } => {
                            in_macro_label_definitions.push(name.clone());
                        }
                        _ => todo!()
                    }
                }
                TokenVariant::Scope => {

                    macro_body.push(token.clone());
                    scope_tracker += 1;
                    continue;
                }


                TokenVariant::Unscope => {

                    macro_body.push(token.clone());
                    scope_tracker -= 1;
                    if scope_tracker != 0 {

                        continue;
                    }
                    /*
                    if let TokenVariant::Linebreak = macro_body[1].variant {
                        macro_body.remove(1);
                    }
                    if let TokenVariant::Linebreak = macro_body[macro_body.len() - 2].variant {
                        macro_body.remove(macro_body.len() - 2);
                    }
                     */

                    let new_macro = Macro {
                        name: macro_name.clone(),
                        info: macro_info.unwrap(),
                        args: macro_args,
                        body: macro_body,
                        labels_defined_in_macro: in_macro_label_definitions,
                    };
                    macros.insert(macro_name, new_macro);
                    macro_body = Vec::new();
                    macro_args = Vec::new();
                    macro_name = String::new();
                    macro_info = None;

                    in_macro_label_definitions = Vec::new();
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
    (new_tokens, macros)
}



fn generate_macro_body(current_macro: &Macro, label_map: &HashMap<String, TokenOrTokenVec>) -> Vec<Token> {
    let mut body: Vec<Token> = Vec::new();
    println_debug!("{:?}", label_map);


    for base_body_token in &current_macro.body {
        match  &base_body_token.variant {
                TokenVariant::Label {name} => {
                let mut n = name.clone();
                if current_macro.labels_defined_in_macro.contains(name) {
                    n = format!("?{}?{}", current_macro.name, name);    // MACRO HYGIENE HACK
                    // Try to just set name maybe?
                }


                let new_token = label_map.get(&n);
                match new_token {
                    Some(t) => {
                        match t {
                            TokenOrTokenVec::Tok(x) => {
                                let mut copy = x.clone();
                                copy.origin_info = Some(base_body_token.info.clone());
                                body.push(copy);
                            }
                            TokenOrTokenVec::TokVec(v) => {
                                for i in v {
                                    let mut copy = i.clone();
                                    copy.origin_info = Some(base_body_token.info.clone());
                                    body.push(copy);
                                }
                            }
                        }
                        continue;
                    }
                    None => {
                        body.push(Token {
                            info: base_body_token.info.clone(),
                            variant: TokenVariant::Label { name: n },
                            origin_info: base_body_token.origin_info.clone()
                        });
                        continue;
                    }
                }
            }
            _ => {
                body.push(base_body_token.clone());

            }
        }


    }


    body
}

#[derive(Debug)]
enum TokenOrTokenVec {
    Tok(Token),
    TokVec(Vec<Token>),
}

fn insert_macros(tokens: Vec<Token>, macros: &HashMap<String, Macro>) -> (bool, Vec<Token>) {
    let mut new_tokens: Vec<Token> = Vec::new();
    enum Mode {
        NORMAL,
        ARGS,
        SCOPED_ARG,
    }
    let mut scope_tracker = 1;

    let mut has_inserted_macro = false;
    let mut mode = Mode::NORMAL;
    let mut current_macro: Option<&Macro> = None;
    let mut label_map: HashMap<String, TokenOrTokenVec> = HashMap::new();

    let mut suffix: Vec<Token> = Vec::new();
    let mut cur_arg_name: String = String::new();
    for token in tokens {
        match mode {
            Mode::NORMAL => match token.variant {
                TokenVariant::MacroCall { name } => {
                    let mac = macros.get(&name);
                    match mac {
                        None => {
                            asm_error!(&token.info, "No declaration found for the macro '{name}'.");
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

                // It has read all arguments
                if label_map.len() >= current_macro_safe.args.len() {
                    let mut body = generate_macro_body(current_macro_safe, &label_map);
                    new_tokens.append(&mut body);
                    new_tokens.append(&mut suffix);
                    suffix = Vec::new();
                    has_inserted_macro = true;
                    mode = Mode::NORMAL;
                    current_macro = None;
                    label_map = HashMap::new();
                    new_tokens.push(token.clone());
                    scope_tracker = 1;

                    continue;
                }
                let name_to_replace = &current_macro_safe.args[label_map.len()];


                if let TokenVariant::Linebreak = token.variant {
                    continue;
                }

                if let TokenVariant::Unscope = token.variant {
                    suffix.push(token.clone());
                    continue;
                }
                let lower = name_to_replace.to_ascii_lowercase();
                match &lower[..2] {
                    x if x == "s_" || x == "m_" => if let TokenVariant::Scope = token.variant {} else { // Change the m_
                        asm_info!(&token.info, "Expected a SCOPE as argument {}", hint!("See the documentation for information on the typing system"));
                        asm_details!(&current_macro_safe.info, "Macro definition");
                    }
                    "l_" => match token.variant {
                        TokenVariant::DecLiteral {..} | TokenVariant::StrLiteral {..} => {}
                        _ => { asm_info!(&token.info, "Expected a LITERAL as argument {}", hint!("See the documentation for information on the typing system"));
                            asm_details!(&current_macro_safe.info, "Macro definition");
                    }
                    }
                    "a_" => {}
                    _ => if let TokenVariant::Label {..} = token.variant {} else {
                        asm_info!(&token.info, "Expected a LABEL as argument, found {:?} {}", &token.variant, hint!("See the documentation for information on the typing system"));
                        asm_details!(&current_macro_safe.info, "Macro definition");

                    }
                }

                if let TokenVariant::Scope = token.variant {
                    mode = Mode::SCOPED_ARG;
                    let toks: Vec<Token> = vec![token.clone()];
                    label_map.insert(name_to_replace.clone(), TokenOrTokenVec::TokVec(toks));
                    cur_arg_name = name_to_replace.clone();

                    continue;
                }


                label_map.insert(name_to_replace.clone(), TokenOrTokenVec::Tok(token.clone()));


                continue;
            }

            Mode::SCOPED_ARG => match token.variant {

                TokenVariant::Scope => {
                    scope_tracker += 1;
                    let tok_vec = label_map.get_mut(&cur_arg_name).unwrap();
                    match tok_vec {
                        TokenOrTokenVec::Tok(x) => todo!(),
                        TokenOrTokenVec::TokVec(v) => {
                            v.push(token);
                        }

                    }
                }
                TokenVariant::Unscope => {
                    scope_tracker -= 1;
                    let tok_vec = label_map.get_mut(&cur_arg_name).unwrap();
                    match tok_vec {
                        TokenOrTokenVec::Tok(x) => todo!(),
                        TokenOrTokenVec::TokVec(v) => {
                            v.push(token);
                        }

                    }
                    if scope_tracker > 0 {
                        continue;
                    }
                    cur_arg_name.clear();
                    mode = Mode::ARGS;

                }
                _ => {
                    let tok_vec = label_map.get_mut(&cur_arg_name).unwrap();
                    match tok_vec {
                        TokenOrTokenVec::Tok(x) => todo!(),
                        TokenOrTokenVec::TokVec(v) => {
                            v.push(token);
                        }

                    }
                }
            }
        }
    }

    (has_inserted_macro, new_tokens)
}

pub fn loop_insert_macros(tokens: Vec<Token>, macros: &HashMap<String, Macro>) -> Vec<Token> {
    let mut has_inserted ;
    let mut t = tokens;
    let mut i = 0;
    loop {
        (has_inserted, t) = insert_macros(t, macros);

        if !has_inserted {
            return t;
        }
        i += 1;
        if i > 100 {
            panic!();
        }
    }
}
