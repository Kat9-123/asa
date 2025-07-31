use crate::args;
use crate::asm_details;
use crate::asm_info;
use crate::feedback::*;
use crate::hint;
use crate::symbols;
use crate::tokens::*;

use colored::Colorize;
use std::collections::HashMap;
use std::fmt;
use std::thread::scope;

#[derive(Debug)]
pub struct Macro {
    name: String,
    info: Info,
    args: Vec<(String, Info)>,
    body: Vec<Token>,
    labels_defined_in_macro: Vec<String>,
}

impl fmt::Display for Macro {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}:    ", self.name.yellow())?;
        for i in &self.args {
            write!(fmt, "{} ", i.0)?;
        }
        write!(fmt, "\n{: >4?}\n", self.body)?;
        Ok(())
    }
}

pub fn read_macros(tokens: Vec<Token>) -> (Vec<Token>, HashMap<String, Macro>) {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut macros: HashMap<String, Macro> = HashMap::new();

    enum Mode {
        Normal,
        Args,
        Body { bounded_by_scopes: bool },
    }
    let mut mode: Mode = Mode::Normal;
    let mut scope_tracker = 0;

    let mut macro_name: String = String::new();
    let mut macro_body: Vec<Token> = Vec::new();
    let mut macro_args: Vec<(String, Info)> = Vec::new();
    let mut macro_info: Option<Info> = None;
    let mut in_macro_label_definitions: Vec<String> = Vec::new();

    for i in 0..tokens.len() {
        let token: &Token = &tokens[i];
        match mode {
            Mode::Normal => match &token.variant {
                TokenVariant::MacroDeclaration { name } => {
                    macro_name = name.clone();

                    if let Some(x) = macros.get(&macro_name) {
                        asm_warn!(
                            &token.info,
                            "A macro with this name has already been defined"
                        );
                        asm_details!(&x.info, "Here");
                    }
                    macro_info = Some(token.info.clone());
                    mode = Mode::Args;
                }
                TokenVariant::MacroBodyStart | TokenVariant::MacroBodyEnd => {
                    asm_error!(&token.info, "Unexpected token");
                }
                _ => {
                    new_tokens.push(token.clone());
                }
            },
            Mode::Args => match &token.variant {
                TokenVariant::Linebreak => { /* Linebreaks are allowed between arguments */ }

                TokenVariant::Label { name } => {
                    macro_args.push((name.clone(), token.info.clone()));
                    if !name.ends_with('?') {
                        asm_info!(
                            &token.info,
                            "Notate macro arguments with a trailing question mark {}",
                            hint!("'{name}' -> '{name}?'")
                        );
                    }
                }
                TokenVariant::MacroBodyStart => {
                    mode = Mode::Body {
                        bounded_by_scopes: false,
                    };
                }
                TokenVariant::Scope => {
                    mode = Mode::Body {
                        bounded_by_scopes: true,
                    };
                    macro_body.push(token.clone());
                    scope_tracker += 1;
                }

                _ => {
                    asm_error!(
                        &token.info,
                        "Only labels may be used as arguments for '{macro_name}'"
                    );
                }
            },
            Mode::Body { bounded_by_scopes } => match &token.variant {
                TokenVariant::LabelArrow { .. } if !bounded_by_scopes => {
                    macro_body.push(token.clone());
                    asm_warn!(
                        &token.info,
                        "Label definitions in non-scoped macros are very dangerous {}",
                        hint!("Use '{{' and '}}' instead of '[' and ']'")
                    );
                }
                // HACK
                TokenVariant::LabelArrow { .. } if bounded_by_scopes => {
                    macro_body.push(token.clone());
                    match &tokens[i - 1].variant {
                        TokenVariant::Label { name } => {
                            in_macro_label_definitions.push(name.clone());
                        }
                        _ => todo!(),
                    }
                }
                TokenVariant::Scope if bounded_by_scopes => {
                    macro_body.push(token.clone());
                    scope_tracker += 1;
                }

                TokenVariant::MacroDeclaration { .. } => {
                    asm_error!(
                        &token.info,
                        "Macros may not be defined inside of other macros"
                    );
                }
                TokenVariant::MacroCall { name } => {
                    if *name == macro_name {
                        asm_error!(&token.info, "Macros may not contain a call to themselves");
                    }
                    macro_body.push(token.clone());
                }

                TokenVariant::MacroBodyEnd if !bounded_by_scopes => {
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
                    mode = Mode::Normal;
                }

                TokenVariant::Unscope if bounded_by_scopes => {
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
                    mode = Mode::Normal;
                }
                _ => {
                    macro_body.push(token.clone());
                }
            },
        }
    }
    (new_tokens, macros)
}

fn generate_macro_body(
    current_macro: &Macro,
    macros: &HashMap<String, Macro>,
    label_map: &HashMap<String, TokenOrTokenVec>,
    context: Vec<(i32, Info)>,
    depth: i32,
) -> Vec<Token> {
    let mut body: Vec<Token> = Vec::new();
    // println_debug!("{:?}", label_map);

    for base_body_token in &current_macro.body {
        match &base_body_token.variant {
            TokenVariant::Label { name } => {
                let mut n = name.clone();
                if current_macro.labels_defined_in_macro.contains(name) {
                    n = format!("?{}?{}", current_macro.name, name); // MACRO HYGIENE HACK
                    // Try to just set name maybe?
                }

                let new_token = label_map.get(&n);
                match new_token {
                    Some(t) => {
                        match t {
                            TokenOrTokenVec::Tok(x) => {
                                let mut copy = x.clone();
                                copy.origin_info = context.clone();
                                copy.origin_info.push((0, base_body_token.info.clone()));

                                body.push(copy);
                            }
                            TokenOrTokenVec::TokVec(v) => {
                                for i in v {
                                    let mut copy = i.clone();
                                    copy.origin_info = context.clone();
                                    copy.origin_info.push((0, base_body_token.info.clone()));

                                    body.push(copy);
                                }
                            }
                        }
                        continue;
                    }
                    None => {
                        let mut origin_info = context.clone();
                        origin_info.push((0, base_body_token.info.clone()));

                        body.push(Token {
                            info: base_body_token.info.clone(),
                            variant: TokenVariant::Label { name: n },
                            origin_info, // macro_trace: macro_trace
                        });
                        continue;
                    }
                }
            }
            _ => {
                let mut c = base_body_token.clone();
                c.origin_info = context.clone();
                c.origin_info.push((0, base_body_token.info.clone()));

                body.push(c);
            }
        }
    }

    //body.push(Token {info: Info {start_char: 0, length: 0, line_number: 0, file: "".to_owned(), }, variant: TokenVariant::Linebreak, origin_info: vec![]});
    // dbg!(&body);

    //dbg!(&macros);
    body = insert_macros(body, macros, depth, context);
    //dbg!(&body);

    body
}

#[derive(Debug)]
enum TokenOrTokenVec {
    Tok(Token),
    TokVec(Vec<Token>),
}

fn macro_argument_type_check(label_to_replace_info: &Info, token: &Token, argument_name: &str) {
    if args::exist() && args::get().disable_type_checking {
        return;
    }
    let lower = argument_name.to_ascii_lowercase();
    if lower.len() > 1 {
        match &lower[..2] {
            x if x == symbols::SCOPE_TYPE_PREFIX || x == symbols::MACRO_TYPE_PREFIX => {
                if let TokenVariant::Scope = token.variant {
                } else {
                    // Change the m_
                    asm_info!(
                        &token.info,
                        "Expected a SCOPE as argument {}",
                        hint!("See the documentation for information on the typing system")
                    );
                    asm_details!(label_to_replace_info, "Macro definition");
                }
                return;
            }
            symbols::LITERAL_TYPE_PREFIX => match token.variant {
                TokenVariant::DecLiteral { .. } | TokenVariant::StrLiteral { .. } => {
                    return;
                }
                _ => {
                    asm_info!(
                        &token.info,
                        "Expected a LITERAL as argument {}",
                        hint!("See the documentation for information on the typing system")
                    );
                    asm_details!(label_to_replace_info, "Macro definition");
                    return;
                }
            },
            symbols::ANY_TYPE_PREFIX => {
                return;
            }
            _ => {}
        }
    }
    if let TokenVariant::Label { .. } = token.variant {
    } else {
        asm_info!(
            &token.info,
            "Expected a LABEL as argument, found {:?} {}",
            &token.variant,
            hint!("See the documentation for information on the typing system")
        );
        asm_details!(label_to_replace_info, "Argument in macro definition");
    }
}

pub fn insert_macros(
    tokens: Vec<Token>,
    macros: &HashMap<String, Macro>,
    depth: i32,
    context: Vec<(i32, Info)>,
) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    #[derive(Debug, PartialEq)]
    enum Mode {
        Normal,
        Args,
        ScopedArg,
    }
    let mut scope_tracker = 1;

    let mut mode = Mode::Normal;
    let mut current_macro: Option<&Macro> = None;
    let mut label_map: HashMap<String, TokenOrTokenVec> = HashMap::new();
    let mut caller_info: Option<Info> = None;
    let mut suffix: Vec<Token> = Vec::new();
    let mut cur_arg_name: String = String::new();
    for token in &tokens {
        match mode {
            Mode::Normal => match &token.variant {
                TokenVariant::MacroCall { name } => {
                    let mac = macros.get(name);
                    match mac {
                        None => {
                            asm_error!(&token.info, "No declaration found for the macro '{name}'.");
                        }
                        Some(x) => {
                            current_macro = Some(x);
                            caller_info = Some(token.info.clone());

                            mode = Mode::Args;
                        }
                    }
                }
                _ => {
                    new_tokens.push(token.clone());
                }
            },
            Mode::Args => {
                let current_macro_safe = current_macro.unwrap();
                // It has read all arguments
                if label_map.len() >= current_macro_safe.args.len() {
                    let mut c = context.clone();
                    c.push((0, caller_info.unwrap()));
                    let mut body =
                        generate_macro_body(current_macro_safe, macros, &label_map, c, depth);
                    new_tokens.append(&mut body);
                    new_tokens.append(&mut suffix);
                    caller_info = None;
                    suffix = Vec::new();
                    mode = Mode::Normal;
                    current_macro = None;
                    label_map = HashMap::new();
                    new_tokens.push(token.clone());

                    scope_tracker = 1;

                    continue;
                }
                let label_to_replace = &current_macro_safe.args[label_map.len()];
                let name_to_replace = &label_to_replace.0;
                let label_to_replace_info = &label_to_replace.1;
                if let TokenVariant::Linebreak = token.variant {
                    continue;
                }
                macro_argument_type_check(label_to_replace_info, token, name_to_replace);

                if let TokenVariant::Scope = token.variant {
                    mode = Mode::ScopedArg;
                    let toks: Vec<Token> = vec![];
                    label_map.insert(name_to_replace.clone(), TokenOrTokenVec::TokVec(toks));
                    cur_arg_name = name_to_replace.clone();
                    scope_tracker = 1;
                    continue;
                }

                label_map.insert(name_to_replace.clone(), TokenOrTokenVec::Tok(token.clone()));
            }

            Mode::ScopedArg => match token.variant {
                TokenVariant::Scope => {
                    scope_tracker += 1;
                    let tok_vec = label_map.get_mut(&cur_arg_name).unwrap();
                    match tok_vec {
                        TokenOrTokenVec::Tok(x) => todo!(),
                        TokenOrTokenVec::TokVec(v) => {
                            v.push(token.clone());
                        }
                    }
                }
                TokenVariant::Unscope => {
                    scope_tracker -= 1;
                    if scope_tracker <= 0 {
                        cur_arg_name.clear();
                        mode = Mode::Args;
                        continue;
                    }

                    let tok_vec = label_map.get_mut(&cur_arg_name).unwrap();
                    match tok_vec {
                        TokenOrTokenVec::Tok(x) => todo!(),
                        TokenOrTokenVec::TokVec(v) => {
                            v.push(token.clone());
                        }
                    }

                }
                _ => {
                    let tok_vec = label_map.get_mut(&cur_arg_name).unwrap();
                    match tok_vec {
                        TokenOrTokenVec::Tok(x) => todo!(),
                        TokenOrTokenVec::TokVec(v) => {
                            v.push(token.clone());
                        }
                    }
                }
            },
        }
    }

    // HACK
    if mode == Mode::Args {
        let current_macro_safe = current_macro.unwrap();
        if current_macro_safe.args.len() != label_map.len() {
            asm_error!(
                &caller_info.unwrap(),
                "Not enough arguments have been supplied"
            );
        }
        // It has read all arguments
        let mut c = context.clone();
        c.push((0, caller_info.unwrap()));
        let mut body = generate_macro_body(current_macro_safe, macros, &label_map, c, depth);
        new_tokens.append(&mut body);
        new_tokens.append(&mut suffix);
    }
    new_tokens
}
