use crate::feedback::*;
use crate::tokens::*;

pub fn char_and_hex_to_dec(tokens: &mut Vec<Token>) {
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

pub fn expand_strings(tokens: Vec<Token>) -> Vec<Token> {
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