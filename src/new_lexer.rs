

use std::{env, result};

use crate::{asm_error,  hint, println_debug, tokens::{Info, LabelOffset, Token}};

#[derive(Debug)]
enum Context {
    None,
    LineComment,
    String,
    Char,

    BlockComment,

    SubleqOrLabelArrow,
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

    MultOrBlockComment,
    PossibleBlockCommentEnd,
}


fn updated_context(context: &Context, buffer: &String, cur_char: char, info: &Info) -> (Context, Option<char>, Option<Token>) {
    match context {
        Context::DontMoveToNextChar => unreachable!(),


        Context::None => match cur_char {
            '\n' => (Context::None, None, Some(Token::Linebreak {info: info.clone()})),
            ' ' => (Context::None, None, None),
            ';' => (Context::LineComment, None, None),
            '-' => (Context::SubleqOrLabelArrow, Some(cur_char), None),
            c if c.is_ascii_digit() => (Context::HexOrDec, Some(c), None),// This will cause hex numbers to have leading zeros
            c if c.is_ascii_alphabetic() || c == '_' || c == '.' => (Context::Label, Some(c), None),

            '*' => (Context::MultOrBlockComment, None, None),

            '{' => (Context::None, None, Some(Token::Scope {info: info.clone()})),
            '}' => (Context::None, None, Some(Token::Unscope {info: info.clone()})),
            '[' => (Context::None, None, Some(Token::MacroBodyStart {info: info.clone()})),
            ']' => (Context::None, None, Some(Token::MacroBodyEnd {info: info.clone()})),
            '\\' => (Context::None, None, Some(Token::Linebreak {info: info.clone()})),

            '/' => (Context::None, None, Some(Token::NamespaceEnd {info: info.clone()})),

            '(' => (Context::None, None, Some(Token::BraceOpen {info: info.clone()})),
            ')' => (Context::None, None, Some(Token::BraceClose {info: info.clone()})),
            '&' => (Context::Relative, None, None),


            '\'' => (Context::Char, None, None),
            '"' => (Context::String, None, None),

            '@' => (Context::MacroDeclaration, None, None),
            '!' => (Context::MacroCall, None, None),
            '#' => (Context::Namespace, None, None),

            _ => asm_error!(info, "Unexpected character"),
        }

        Context::MultOrBlockComment => match cur_char {
            '*' => (Context::BlockComment, None, None),
            _ => (Context::DontMoveToNextChar, None, Some(Token::Mult {info: info.clone()}))
        }

        Context::BlockComment => match cur_char {
            '*' => (Context::PossibleBlockCommentEnd, None, None),
            _ => (Context::BlockComment, None, None),
        }
        Context::PossibleBlockCommentEnd => match cur_char {
            '*' => (Context::None, None, None,),
            _ => (Context::BlockComment, None, None)
        }

        Context::LineComment => match cur_char {
            '\n' => (Context::DontMoveToNextChar, None, None),
            _ => (Context::LineComment, None, None),
        }
        Context::Namespace => match cur_char {
            '\n' => (Context::DontMoveToNextChar, None, Some(Token::Namespace { info: info.clone(), name: buffer.clone() })),
            _ => (Context::Namespace, Some(cur_char), None)
        }


        Context::SubleqOrLabelArrow => match cur_char {     // Negative numbers :(
            '=' => (Context::None, None, Some(Token::Subleq {info: info.clone()})),

            'a' | 'b' | 'c' => (Context::LabelArrow, Some(cur_char), None),
            '>' => (Context::None, Some(cur_char), Some(Token::LabelArrow {info: info.clone(), offset: LabelOffset::Int(0)})),
            _ => { asm_error!(info, "Unexpected character, for Subleq use '-=', for label use ->") },
        }

        Context::LabelArrow => match cur_char {
            '>' => {
                let ch: char = buffer.chars().nth(1).unwrap();
                if ch == 'a' || ch == 'b' || ch == 'c' {
                    return (Context::None, Some(cur_char), Some(Token::LabelArrow {info: info.clone(), offset: LabelOffset::Char(ch)}));
                }
                asm_error!(info, "Unexpected character");
            }
            _ => todo!()
        }


        Context::HexOrDec => match cur_char {
            'x' => (Context::Hex, None, None),
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            c if c.is_ascii_alphabetic() => asm_error!(info, "Unexpected character when defining Hex or Dec literal {}", hint!("Labels may not start with a number")),
            _ => (Context::DontMoveToNextChar, None, Some(Token::DecLiteral { info: info.clone(), value: buffer.parse::<i32>().unwrap() })),
        }

        Context::Hex => match cur_char {
            c if c.is_ascii_hexdigit() => (Context::Hex, Some(c), None),
            c if c.is_ascii_alphabetic() => asm_error!(info, "Unexpected character when defining Hex literal"),
            _ => (Context::DontMoveToNextChar, None, Some(Token::HexLiteral {info: info.clone(),  value: buffer.clone() })),    // may cause issues
        }

        Context::Dec => match cur_char {
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            c if c.is_ascii_alphabetic() => asm_error!(info, "Unexpected character when defining Dec literal"),
            _ => (Context::DontMoveToNextChar, None, Some(Token::DecLiteral { info: info.clone(), value: buffer.parse::<i32>().unwrap() })),
        }
        Context::Label => match cur_char {
            '\n' => (Context::DontMoveToNextChar, None, Some(Token::Label { info: info.clone(), name: buffer.clone() })),
            c if c.is_alphanumeric() || c == '?' || c == '_'  || c == ':' || c == '.' => (Context::Label, Some(cur_char), None),
            _ => (Context::DontMoveToNextChar, None, Some(Token::Label { info: info.clone(), name: buffer.clone() })),
        }

        Context::MacroDeclaration => match cur_char {
            ' ' | '\n' => (Context::DontMoveToNextChar, None,Some(Token::MacroDeclaration { info: info.clone(), name: buffer.clone() })),
            _ => (Context::MacroDeclaration, Some(cur_char), None),
        }
        Context::MacroCall => match cur_char {
            ' ' | '\n' => (Context::DontMoveToNextChar, None,Some(Token::MacroCall { info: info.clone(), name: buffer.clone() })),
            _ => (Context::MacroCall, Some(cur_char), None),
        }
        Context::Char => match cur_char {   // Not great
            '\'' => (Context::None, None, Some(Token::CharLiteral { info: info.clone(), value: buffer.chars().nth(0).unwrap() })),
            _ => (Context::Char, Some(cur_char), None),
        }
        Context::String => match cur_char {
            '"' => (Context::None, None, Some(Token::StrLiteral {info: info.clone(),  value: buffer.clone() })),
            _ => (Context::String, Some(cur_char), None),
        }
        Context::Relative => match cur_char {
            c if c.is_ascii_digit() => (Context::Relative, Some(c), None),
            _ => {
                let mut offset= 1;
                if buffer != "" {
                    offset =  buffer.parse::<i32>().unwrap();
                }
                (Context::DontMoveToNextChar, None, Some(Token::Relative { info: info.clone(), offset  }))
            }

        }

    }

}

pub fn clean(text: String) -> String {
    let cleaned_string: String = text.replace("\r\n", "\n").replace("\t", " ");
    return cleaned_string;
}

pub fn tokenise(mut text: String, path: String) -> Vec<Token> {
    text = clean(text);
    text.push('\n');

    let mut name_space_stack : Vec<String> = vec![path.clone()];
    let mut line_number_stack: Vec<i32> = Vec::new();
    let mut result_tokens: Vec<Token> = Vec::new();

    let mut context: Context = Context::None;
    let mut buffer: String = String::new();
    let mut info: Info = Info { start_char: 0, length: 0, line_number: 1, file: path };

    for c in text.chars() {
        loop {

            let (new_context, add_to_buffer, token_to_add) = updated_context(&context, &buffer, c, &info);
            println_debug!("{:?}, {:?}, {:?}", new_context, add_to_buffer, token_to_add);

            context = new_context;
            match add_to_buffer {
                Some(ch) => buffer.push(ch),
                None => {}
            }
            info.length = buffer.len() as i32 + 1;

            match token_to_add {
                Some(tok) => {

                    if let Token::Linebreak { .. } = tok  {

                        info.line_number += 1;
                        info.start_char = 0;
                        info.length = 1;

                    } else {
                        info.start_char += (info.length - 1);
                    }

                    if let Token::Namespace { info: _info, name  } = &tok {
                        name_space_stack.push(name.clone());
                        line_number_stack.push(info.line_number);
                        info.line_number = 0;
                        info.file = name.clone();
                    }
                    if let Token::NamespaceEnd { info: _info } = &tok {
                        name_space_stack.pop();
                        info.file = name_space_stack.last().unwrap().clone();
                        info.line_number = line_number_stack.pop().unwrap();
                    }

                    result_tokens.push(tok);

                    buffer.clear();
                }
                None => {}
            }

            if let Context::DontMoveToNextChar = context {
                context = Context::None;
                continue;
            }
            if buffer == "" {
                info.start_char += 1;
            }


            break;
        }


    }


    return result_tokens;
}

