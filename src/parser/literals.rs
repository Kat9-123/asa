use crate::feedback::*;
use crate::tokens::*;
use unescape::unescape;

pub fn char_and_hex_to_dec(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        match &token.variant {
            TokenVariant::HexLiteral { value } => {
                let val = match i32::from_str_radix(value, 16) {
                    Err(_) => asm_error!(&token.info, "Invalid hex literal"),
                    Ok(x) => x,
                };

                token.variant = TokenVariant::DecLiteral { value: val };
            }
            TokenVariant::CharLiteral { value } => {
                token.variant = TokenVariant::DecLiteral {
                    value: *value as i32,
                };
            }
            _ => continue,
        }
    }
}

pub fn convert_strings(tokens: Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();
    for token in tokens {
        match token.variant {
            TokenVariant::StrLiteral { value } => {
                let string = unescape(&value).unwrap();
                for c in string.chars() {
                    new_tokens.push(Token {
                        info: token.info.clone(),
                        origin_info: token.origin_info.clone(),
                        variant: TokenVariant::DecLiteral { value: c as i32 },
                    });
                }
            }
            _ => new_tokens.push(token),
        }
    }

    new_tokens
}
