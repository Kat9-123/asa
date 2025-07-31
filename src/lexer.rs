//! Converts a string into a vector of tokens.

use colored::Colorize;

use crate::{
    asm_error, asm_hint, preprocessor,
    tokens::{Info, LabelOffset, Token, TokenVariant},
};

#[derive(Debug, PartialEq, Eq)]
enum Context {
    None,
    LineComment,
    String,
    Char,

    BlockComment,

    SubleqOrNegativeOrLabelArrow,
    LabelArrow,
    HexOrDec,
    Hex,
    Dec,

    Label,
    DontMoveToNextChar,

    MacroDeclaration,
    MacroCall,
    Relative,

    Namespace,

    AsteriskOrBlockComment,
    PossibleBlockCommentEnd,
}

fn updated_context(
    context: &Context,
    buffer: &String,
    cur_char: char,
    info: &Info,
) -> (Context, Option<char>, Option<TokenVariant>) {
    match context {
        Context::DontMoveToNextChar => unreachable!(),

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
            '\\' => (Context::None, None, Some(TokenVariant::Linebreak)),

            '/' => (Context::None, None, Some(TokenVariant::NamespaceEnd)),

            '(' => (Context::None, None, Some(TokenVariant::BraceOpen)),
            ')' => (Context::None, None, Some(TokenVariant::BraceClose)),
            '&' => (Context::Relative, None, None),

            '\'' => (Context::Char, None, None),
            '"' => (Context::String, None, None),

            '@' => (Context::MacroDeclaration, None, None),
            '!' => (Context::MacroCall, None, None),
            '#' => (Context::Namespace, None, None),

            '?' => asm_error!(
                info,
                "Unexpected character {}",
                asm_hint!("Labels may not start with a '?'")
            ),
            _ => asm_error!(info, "Unexpected character"),
        },

        Context::AsteriskOrBlockComment => match cur_char {
            '*' => (Context::BlockComment, None, None),
            _ => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::Asterisk),
            ),
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
            '\n' => (Context::DontMoveToNextChar, None, None),
            _ => (Context::LineComment, None, None),
        },
        Context::Namespace => match cur_char {
            '\n' => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::Namespace {
                    name: buffer.clone(),
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
            c if c.is_ascii_alphabetic() => asm_error!(
                info,
                "Unexpected character when defining Hex or Dec literal {}",
                asm_hint!("Labels may not start with a number")
            ),
            _ => (
                Context::DontMoveToNextChar,
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
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::HexLiteral {
                    value: buffer.clone(),
                }),
            ), // may cause issues
        },

        Context::Dec => match cur_char {
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            c if c.is_ascii_alphabetic() => {
                asm_error!(info, "Unexpected character when defining Dec literal")
            }
            _ => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::DecLiteral {
                    value: buffer.parse::<i32>().unwrap(),
                }),
            ),
        },
        Context::Label => match cur_char {
            '\n' => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::Label {
                    name: buffer.clone(),
                }),
            ),
            c if c.is_alphanumeric() || c == '?' || c == '_' || c == ':' || c == '.' => {
                (Context::Label, Some(cur_char), None)
            }
            _ => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::Label {
                    name: buffer.clone(),
                }),
            ),
        },

        Context::MacroDeclaration => match cur_char {
            c if c.is_alphanumeric() || c == '_' || c == ':' => {
                (Context::MacroDeclaration, Some(cur_char), None)
            }

            _ => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::MacroDeclaration {
                    name: buffer.clone(),
                }),
            ),
        },
        Context::MacroCall => match cur_char {
            ' ' | '\n' => (
                Context::DontMoveToNextChar,
                None,
                Some(TokenVariant::MacroCall {
                    name: buffer.clone(),
                }),
            ),
            _ => (Context::MacroCall, Some(cur_char), None),
        },
        Context::Char => match cur_char {
            // Not great
            '\'' => (
                Context::None,
                None,
                Some(TokenVariant::CharLiteral {
                    value: buffer.chars().next().unwrap(),
                }),
            ),
            _ => (Context::Char, Some(cur_char), None),
        },
        Context::String => match cur_char {
            '"' => (
                Context::None,
                None,
                Some(TokenVariant::StrLiteral {
                    value: buffer.clone(),
                }),
            ),
            _ => (Context::String, Some(cur_char), None),
        },

        Context::Relative => match cur_char {
            c if c.is_ascii_digit() => (Context::Relative, Some(c), None),
            c => {
                let mut offset = 1;
                if !buffer.is_empty() {
                    offset = buffer.parse::<i32>().unwrap();
                    if c == 'x' && buffer.starts_with('0') {
                        asm_error!(
                            info,
                            "& may not directly precede a hex number {}",
                            asm_hint!(
                                "Place a space in between & and the Hex number. '&0x...' -> '& 0x...'"
                            )
                        );
                    }
                }

                (
                    Context::DontMoveToNextChar,
                    None,
                    Some(TokenVariant::Relative { offset }),
                )
            }
        },
    }
}

pub fn tokenise(mut text: String, path: String) -> Vec<Token> {
    text = preprocessor::generic_sanitisation(&text);
    text.push('\n'); // Little hack

    let mut name_space_stack: Vec<String> = vec![path.clone()];
    let mut line_number_stack: Vec<i32> = Vec::new();
    let mut result_tokens: Vec<Token> = Vec::new();

    let mut context: Context = Context::None;
    let mut buffer: String = String::new();
    let mut info: Info = Info {
        start_char: 0,
        length: 0,
        line_number: 1,
        file: path,
    };
    let mut idx_in_line = 0;
    for c in text.chars() {
        loop {
            let (new_context, add_to_buffer, variant_to_add) =
                updated_context(&context, &buffer, c, &info);

            context = new_context;
            info.start_char = idx_in_line - info.length;

            //println_debug!("{idx_in_line} '{c}' {:?}, {:?}, {:?}, {:?}", context, add_to_buffer, variant_to_add, info);

            if let Some(ch) = add_to_buffer {
                buffer.push(ch)
            }

            if let Some(var) = &variant_to_add {
                match &var {
                    TokenVariant::Namespace { name } => {
                        name_space_stack.push(name.clone());
                        line_number_stack.push(info.line_number);
                        info.line_number = 0;
                        info.file = name.clone();
                    }
                    TokenVariant::NamespaceEnd => {
                        name_space_stack.pop();
                        info.file = name_space_stack.last().unwrap().clone();
                        info.line_number = line_number_stack.pop().unwrap();
                    }
                    _ => {}
                }
                let token = Token {
                    info: info.clone(),
                    variant: var.clone(),
                    origin_info: vec![], //macro_trace: None,
                };
                //   println!("{:?} {:?}", token, token.info);
                result_tokens.push(token);

                buffer.clear();
                if let TokenVariant::Linebreak = var {
                    info.line_number += 1;
                    info.start_char = 0;
                    info.length = 0;
                    idx_in_line = 0;
                    break;
                }
            }

            if let Context::DontMoveToNextChar = context {
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
