

use std::result;

use crate::tokens::{LabelOffset, Token};

#[derive(Debug)]
enum Context {
    None,
    LineComment,
    String,
    Char,


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
}


fn updated_context(context: &Context, buffer: &String, cur_char: char) -> (Context, Option<char>, Option<Token>) {
    match context {
        Context::DontMoveToNextChar => unreachable!(),


        Context::None => match cur_char {
            '\n' => (Context::None, None, Some(Token::Linebreak)),
            ' ' => (Context::None, None, None),
            ';' => (Context::LineComment, None, None),
            '-' => (Context::SubleqOrLabelArrow, None, None),
            c if c.is_ascii_digit() => (Context::HexOrDec, Some(c), None),// This will cause hex numbers to have leading zeros
            c if c.is_ascii_alphabetic() || c == '_' => (Context::Label, Some(c), None),

            '{' => (Context::None, None, Some(Token::Scope)),
            '}' => (Context::None, None, Some(Token::Unscope)),
            '[' => (Context::None, None, Some(Token::MacroBodyStart)),
            ']' => (Context::None, None, Some(Token::MacroBodyEnd)),
            '\\' => (Context::None, None, Some(Token::Linebreak)),

            '(' => (Context::None, None, Some(Token::BraceOpen)),
            ')' => (Context::None, None, Some(Token::BraceClose)),
            '&' => (Context::Relative, None, None),


            '\'' => (Context::Char, None, None),
            '"' => (Context::String, None, None),

            '@' => (Context::MacroDeclaration, None, None),
            '!' => (Context::MacroCall, None, None),
            '#' => (Context::Namespace, None, None),

            _ => (Context::None, None, None)
        }
        Context::LineComment => match cur_char {
            '\n' => (Context::DontMoveToNextChar, None, None),
            _ => (Context::LineComment, None, None),
        }
        Context::Namespace => match cur_char {
            '\n' => (Context::DontMoveToNextChar, None, Some(Token::Namespace { name: buffer.clone() })),
            _ => (Context::Namespace, Some(cur_char), None)
        }


        Context::SubleqOrLabelArrow => match cur_char {     // Negative numbers :(
            '=' => (Context::None, None, Some(Token::Subleq)),

            'a' | 'b' | 'c' => (Context::LabelArrow, Some(cur_char), None),
            '>' => (Context::None, None, Some(Token::LabelArrow {offset: LabelOffset::Int(0)})),
            _ => todo!()
        }

        Context::LabelArrow => match cur_char {
            '>' => {
                let ch: char = buffer.chars().nth(0).unwrap();
                if ch == 'a' || ch == 'b' || ch == 'c' {
                    return (Context::None, None, Some(Token::LabelArrow {offset: LabelOffset::Char(ch)}));
                }
                todo!(); // Numerical offsets.
            }
            _ => todo!()
        }


        Context::HexOrDec => match cur_char {
            'x' => (Context::Hex, None, None),
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            ' ' | '\n' => (Context::DontMoveToNextChar, None, Some(Token::DecLiteral { value: buffer.parse::<i32>().unwrap() })),
            _ => todo!()
        }

        Context::Hex => match cur_char {
            c if c.is_ascii_hexdigit() => (Context::Hex, Some(c), None),
            _ => (Context::DontMoveToNextChar, None, Some(Token::HexLiteral { value: buffer.clone() })),    // may cause issues
        }

        Context::Dec => match cur_char {
            c if c.is_ascii_digit() => (Context::Dec, Some(c), None),
            ' ' | '\n' => (Context::DontMoveToNextChar, None, Some(Token::DecLiteral { value: buffer.parse::<i32>().unwrap() })),
            _=> todo!()
        }
        
        Context::Label => match cur_char {
            '\n' => (Context::DontMoveToNextChar, None, Some(Token::Label { name: buffer.clone() })),
            c if c.is_alphanumeric() || c == '?' || c == '_' => (Context::Label, Some(cur_char), None),
            _ => (Context::DontMoveToNextChar, None, Some(Token::Label { name: buffer.clone() })),
        }

        Context::MacroDeclaration => match cur_char {
            ' ' | '\n' => (Context::DontMoveToNextChar, None,Some(Token::MacroDeclaration { name: buffer.clone() })),
            _ => (Context::MacroDeclaration, Some(cur_char), None),
        }
        Context::MacroCall => match cur_char {
            ' ' | '\n' => (Context::DontMoveToNextChar, None,Some(Token::MacroCall { name: buffer.clone() })),
            _ => (Context::MacroCall, Some(cur_char), None),
        }
        Context::Char => match cur_char {   // Not great
            '\'' => (Context::None, None, Some(Token::CharLiteral { value: buffer.chars().nth(0).unwrap() })),
            _ => (Context::Char, Some(cur_char), None),
        }
        Context::String => match cur_char {
            '"' => (Context::None, None, Some(Token::StrLiteral { value: buffer.clone() })),
            _ => (Context::String, Some(cur_char), None),
        }
        Context::Relative => match cur_char {
            c if c.is_ascii_digit() => (Context::Relative, Some(c), None),
            ' ' | '\n' => (Context::DontMoveToNextChar, None, Some(Token::Relative { offset: buffer.parse::<i32>().unwrap() })),
            _ => todo!()
        }

    }

}

pub fn clean(text: String) -> String {
    let cleaned_string: String = text.replace("\r\n", "\n").replace("\t", " ");
    return cleaned_string;
}

pub fn tokenise(mut text: String) -> Vec<Token> {
    text = clean(text);
    text.push('\n');

    let mut result_tokens: Vec<Token> = Vec::new();

    let mut context: Context = Context::None;
    let mut buffer: String = String::new();

    for c in text.chars() {
        loop {

            let (new_context, add_to_buffer, token_to_add) = updated_context(&context, &buffer, c);
            println!("{:?}, {:?}, {:?}", new_context, add_to_buffer, token_to_add);

            context = new_context;
            match add_to_buffer {
                Some(ch) => buffer.push(ch),
                None => {}
            }

            match token_to_add {
                Some(tok) => {
                    result_tokens.push(tok);
                    buffer.clear();
                }
                None => {}
            }
            if let Context::DontMoveToNextChar = context {
                context = Context::None;
                continue;
            }
            break;
        }


    }


    return result_tokens;
}

