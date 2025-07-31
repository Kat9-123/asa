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
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_char_hex_to_dec() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::CharLiteral { value: '~' },
            TokenVariant::CharLiteral { value: '\0' },
            TokenVariant::CharLiteral { value: 'P' },
            TokenVariant::HexLiteral {
                value: "8000".to_string(),
            },
            TokenVariant::HexLiteral {
                value: "0123".to_string(),
            },
            TokenVariant::HexLiteral {
                value: "-0AA".to_string(),
            },
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::DecLiteral { value: 126 },
            TokenVariant::DecLiteral { value: 0 },
            TokenVariant::DecLiteral { value: 80 },
            TokenVariant::DecLiteral { value: 32768 },
            TokenVariant::DecLiteral { value: 291 },
            TokenVariant::DecLiteral { value: -170 },
        ]);
        char_and_hex_to_dec(&mut input);
        assert_eq!(input, expected);
    }

    #[test]
    fn test_convert_strings() {
        let input: Vec<Token> = tokens_from_token_variant_vec(vec![TokenVariant::StrLiteral {
            value: "Hello, \t\nW0rld!".to_string(),
        }]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::DecLiteral { value: 72 },
            TokenVariant::DecLiteral { value: 101 },
            TokenVariant::DecLiteral { value: 108 },
            TokenVariant::DecLiteral { value: 108 },
            TokenVariant::DecLiteral { value: 111 },
            TokenVariant::DecLiteral { value: 44 },
            TokenVariant::DecLiteral { value: 32 },
            TokenVariant::DecLiteral { value: 9 },
            TokenVariant::DecLiteral { value: 10 },
            TokenVariant::DecLiteral { value: 87 },
            TokenVariant::DecLiteral { value: 48 },
            TokenVariant::DecLiteral { value: 114 },
            TokenVariant::DecLiteral { value: 108 },
            TokenVariant::DecLiteral { value: 100 },
            TokenVariant::DecLiteral { value: 33 },
        ]);

        let output = convert_strings(input);
        assert_eq!(output, expected);
    }
}
