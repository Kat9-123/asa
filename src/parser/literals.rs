use crate::feedback::*;
use crate::tokens::*;

pub fn char_and_hex_to_dec(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        match token {
            Token::HexLiteral {info, value } => {
                *token = Token::DecLiteral {
                    info: info.clone(),
                    value: i32::from_str_radix(value, 16).expect("Should be hex."),
                };
            }
            Token::CharLiteral { info, value } => {
                *token = Token::DecLiteral {
                    info: info.clone(),

                    value: *value as i32
                };
            }
            _ => continue,
        }
    }
}

pub fn expand_strings(tokens: Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();
    for token in tokens {
        match token {
            Token::StrLiteral {info, value } => {
                for c in value.chars() {
                    new_tokens.push(Token::CharLiteral {info: info.clone(), value: c });
                }
            }
            _ => new_tokens.push(token)
        }
    }


    return new_tokens;
}