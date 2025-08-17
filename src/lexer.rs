//! Converts a string into a vector of tokens.

use std::cell::RefCell;

use crate::terminate;
use crate::{
    asm_error, asm_error_no_terminate, asm_hint, preprocessor,
    tokens::{Info, LabelOffset, Token, TokenVariant},
};
use colored::Colorize;

use unescape::unescape;

thread_local! {
    pub static FILES: RefCell<Vec<String>> = const { RefCell::new(vec![]) };
}

#[derive(Debug, PartialEq, Eq)]
enum Context {
    None,
    LineComment,
    String,
    Char,
    EscapedChar,
    BlockComment,

    SubleqOrNegativeOrLabelArrow,
    LabelArrow,
    HexOrDec,
    Hex,
    Dec,

    Label,
    DontConsume,

    MacroDeclaration,
    MacroCall,
    Relative,

    Namespace,

    AsteriskOrBlockComment,
    PossibleBlockCommentEnd,
}

fn is_valid_macro_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ':'
}
fn is_valid_label_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ':' || c == '?' || c == '.'
}
fn updated_context(
    context: &Context,
    buffer: &str,
    cur_char: char,
    #[allow(unused_variables)] info: &Info, // Maybe I'm missing something, but this variable is most definitely used.
) -> (Context, Option<char>, Option<TokenVariant>) {
    match context {
        Context::DontConsume => unreachable!(),

        Context::None => match cur_char {
            '\n' => (Context::None, None, Some(TokenVariant::Linebreak)),
            ' ' => (Context::None, None, None),
            ';' => (Context::LineComment, None, None),
            '-' => (Context::SubleqOrNegativeOrLabelArrow, Some(cur_char), None),
            '0' => (Context::HexOrDec, Some(cur_char), None), // This will cause hex numbers to have leading zeros
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            c if c.is_ascii_alphabetic() || c == '_' || c == '.' => (Context::Label, Some(c), None),

            '*' => (Context::AsteriskOrBlockComment, None, None),

            '{' => (Context::None, None, Some(TokenVariant::Scope)),
            '}' => (Context::None, None, Some(TokenVariant::Unscope)),
            '[' => (Context::None, None, Some(TokenVariant::MacroBodyStart)),
            ']' => (Context::None, None, Some(TokenVariant::MacroBodyEnd)),
            '=' => (Context::None, None, Some(TokenVariant::Equals)),

            '/' => (Context::None, None, Some(TokenVariant::NamespaceEnd)),

            '(' => (Context::None, None, Some(TokenVariant::BraceOpen)),
            ')' => (Context::None, None, Some(TokenVariant::BraceClose)),
            '$' => (Context::Relative, None, None),
            '&' => (
                Context::None,
                None,
                Some(TokenVariant::Relative { offset: 1 }),
            ),
            '\'' => (Context::Char, None, None),
            '"' => (Context::String, None, None),

            '@' => (Context::MacroDeclaration, None, None),
            '!' => (Context::MacroCall, None, None),
            '#' => (Context::Namespace, None, None),

            '?' => {
                asm_error_no_terminate!(info, "Unexpected character");
                asm_hint!("Labels may not start with a '?'");
                terminate!();
            }
            _ => asm_error!(info, "Unexpected character"),
        },

        Context::AsteriskOrBlockComment => match cur_char {
            '*' => (Context::BlockComment, None, None),
            _ => (Context::DontConsume, None, Some(TokenVariant::Asterisk)),
        },

        Context::BlockComment => match cur_char {
            '*' => (Context::PossibleBlockCommentEnd, None, None),
            '\n' => (Context::BlockComment, None, Some(TokenVariant::Linebreak)),
            _ => (Context::BlockComment, None, None),
        },
        Context::PossibleBlockCommentEnd => match cur_char {
            '*' => (Context::None, None, None),
            _ => (Context::BlockComment, None, None),
        },

        Context::LineComment => match cur_char {
            '\n' => (Context::DontConsume, None, None),
            _ => (Context::LineComment, None, None),
        },
        Context::Namespace => match cur_char {
            '\n' => (
                Context::DontConsume,
                None,
                Some(TokenVariant::Namespace {
                    name: buffer.to_owned(),
                }),
            ),
            _ => (Context::Namespace, Some(cur_char), None),
        },

        Context::SubleqOrNegativeOrLabelArrow => match cur_char {
            '=' => (Context::None, None, Some(TokenVariant::Subleq)),
            '0' => (Context::HexOrDec, Some(cur_char), None),
            c if c.is_ascii_digit() => (Context::Dec, Some(cur_char), None),
            'a' | 'b' | 'c' => (Context::LabelArrow, Some(cur_char), None),
            '>' => (
                Context::None,
                Some(cur_char),
                Some(TokenVariant::LabelArrow {
                    offset: LabelOffset::Int(0),
                }),
            ),
            _ => {
                asm_error!(
                    info,
                    "Unexpected character, for Subleq use '-=', for label use ->"
                )
            }
        },

        Context::LabelArrow => match cur_char {
            '>' => {
                let ch: char = buffer.chars().nth(1).unwrap();
                if ch == 'a' || ch == 'b' || ch == 'c' {
                    return (
                        Context::None,
                        Some(cur_char),
                        Some(TokenVariant::LabelArrow {
                            offset: LabelOffset::Char(ch),
                        }),
                    );
                }
                asm_error!(info, "Unexpected character");
            }
            _ => asm_error!(info, "Unexpected character"),
        },

        Context::HexOrDec => match cur_char {
            'x' => (Context::Hex, None, None),
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            c if c.is_ascii_alphabetic() => {
                asm_error_no_terminate!(
                    info,
                    "Unexpected character when defining Hex or Dec literal",
                );
                asm_hint!("Labels may not start with a number");
                terminate!();
            }
            _ => (
                Context::DontConsume,
                None,
                Some(TokenVariant::DecLiteral {
                    value: buffer.parse::<i32>().unwrap(),
                }),
            ),
        },

        Context::Hex => match cur_char {
            c if c.is_ascii_hexdigit() => (Context::Hex, Some(c), None),
            c if c.is_ascii_alphabetic() => {
                asm_error!(info, "Unexpected character when defining Hex literal")
            }
            _ => (
                Context::DontConsume,
                None,
                Some(TokenVariant::HexLiteral {
                    value: buffer.to_owned(),
                }),
            ), // may cause issues
        },

        Context::Dec => match cur_char {
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            c if c.is_ascii_alphabetic() => {
                asm_error!(info, "Unexpected character when defining Dec literal")
            }
            _ => (
                Context::DontConsume,
                None,
                Some(TokenVariant::DecLiteral {
                    value: buffer.parse::<i32>().unwrap(),
                }),
            ),
        },
        Context::Label => match cur_char {
            c if is_valid_label_name(c) => (Context::Label, Some(cur_char), None),
            _ => (
                Context::DontConsume,
                None,
                Some(TokenVariant::Label {
                    name: buffer.to_owned(),
                }),
            ),
        },

        Context::MacroDeclaration => match cur_char {
            c if is_valid_macro_name(c) => (Context::MacroDeclaration, Some(cur_char), None),

            _ => (
                Context::DontConsume,
                None,
                Some(TokenVariant::MacroDeclaration {
                    name: buffer.to_owned(),
                }),
            ),
        },
        Context::MacroCall => match cur_char {
            c if is_valid_macro_name(c) => (Context::MacroCall, Some(cur_char), None),
            _ => (
                Context::DontConsume,
                None,
                Some(TokenVariant::MacroCall {
                    name: buffer.to_owned(),
                }),
            ),
        },
        Context::Char => match cur_char {
            // Not great
            '\\' => (Context::EscapedChar, None, None),
            '\'' => (
                Context::None,
                None,
                Some(TokenVariant::CharLiteral {
                    value: buffer.chars().next().unwrap(),
                }),
            ),
            _ => (Context::Char, Some(cur_char), None),
        },
        Context::EscapedChar => match cur_char {
            '\'' => (
                Context::None,
                None,
                Some(TokenVariant::CharLiteral {
                    value: unescape(&format!("\\{}", buffer.chars().next().unwrap()))
                        .unwrap_or_else(|| asm_error!(&info, "Invalid escape sequence"))
                        .chars()
                        .next()
                        .unwrap(),
                }),
            ),
            _ => (Context::EscapedChar, Some(cur_char), None),
        },
        Context::String => match cur_char {
            '"' => (
                Context::None,
                None,
                Some(TokenVariant::StrLiteral {
                    value: buffer.to_owned(),
                }),
            ),
            _ => (Context::String, Some(cur_char), None),
        },

        Context::Relative => match cur_char {
            c if c.is_ascii_digit() => (Context::Relative, Some(c), None),
            c => {
                let offset = if !buffer.is_empty() {
                    if c == 'x' && buffer.starts_with('0') {
                        asm_error!(info, "Hex numbers may not be used as relative offsets");
                    }
                    buffer.parse::<i32>().unwrap()
                } else {
                    asm_error!(info, "Expected an offset",);
                };

                (
                    Context::DontConsume,
                    None,
                    Some(TokenVariant::Relative { offset }),
                )
            }
        },
    }
}

pub fn tokenise(mut text: String, path: String) -> Vec<Token> {
    let mut result_tokens: Vec<Token> = Vec::new();

    text = preprocessor::generic_sanitisation(&text);
    text.push('\n'); // Little hack

    let mut name_space_stack: Vec<usize> = vec![0];
    FILES.set(vec![path.clone()]);
    let mut line_number_stack: Vec<i32> = Vec::new();

    let mut context: Context = Context::None;
    let mut buffer: String = String::new();
    let mut info: Info = Info {
        start_char: 0,
        length: 0,
        line_number: 1,
        file: 0,
        sourceline_suffix: None,
    };
    let mut idx_in_line = 1;
    for c in text.chars() {
        loop {
            let (new_context, add_to_buffer, variant_to_add) =
                updated_context(&context, &buffer, c, &info);

            context = new_context;
            info.start_char = idx_in_line - info.length;

            if let Some(ch) = add_to_buffer {
                buffer.push(ch)
            }

            if let Some(var) = &variant_to_add {
                match &var {
                    TokenVariant::Namespace { name } => {
                        let idx = FILES.with_borrow_mut(|files| {
                            files.iter().position(|r| r == name).unwrap_or_else(|| {
                                files.push(name.clone());
                                files.len() - 1
                            })
                        });

                        name_space_stack.push(idx);
                        line_number_stack.push(info.line_number);
                        info.line_number = 0;
                        info.file = idx;
                    }
                    TokenVariant::NamespaceEnd => {
                        name_space_stack.pop();
                        info.file = *name_space_stack.last().expect("Unmatched namespace symbol");
                        info.line_number = line_number_stack.pop().unwrap();
                    }
                    _ => {}
                }

                if context == Context::None {
                    info.length += 1;
                }
                let token = Token {
                    info: info.clone(),
                    variant: var.clone(),
                    origin_info: vec![],
                };

                // Consecutive newlines do not carry any information
                if let Some(x) = result_tokens.last() {
                    if token.variant != TokenVariant::Linebreak
                        || x.variant != TokenVariant::Linebreak
                    {
                        result_tokens.push(token);
                    }
                } else {
                    result_tokens.push(token);
                }

                buffer.clear();
                if let TokenVariant::Linebreak = var {
                    info.line_number += 1;
                    info.start_char = 0;
                    info.length = 0;
                    idx_in_line = 0;
                    break;
                }
            }

            if let Context::DontConsume = context {
                context = Context::None;
                continue;
            }

            break;
        }

        if context == Context::None {
            info.length = 0;
        } else {
            info.length += 1;
        }
        idx_in_line += 1;
    }

    result_tokens
}
