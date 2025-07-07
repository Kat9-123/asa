use crate::tokens::Token;

pub fn clean(text: String) -> String {
    let cleaned_string: String = text.replace("\r\n", "\n").replace("\t", " ");
    return cleaned_string;
}

fn match_token(token_str: &str) -> Token {
    let token_len = token_str.len();
    // Dec Literals
    match token_str.parse::<i32>() {
        Ok(val) => return Token::DecLiteral { value: val },
        Err(_) => (),
    }
    // Hex literals
    if token_str.len() > 2 {
        match &token_str[..2] {
            "0x" => {
                return Token::HexLiteral {
                    value: (token_str[2..]).to_string(),
                };
            }
            _ => (),
        }
    }

    if token_str == "-=" {
        return Token::Subleq;
    }

    // Pointers
    if token_str == "->" {
        return Token::LabelArrow;
    }

    if token_str == "[" {
        return Token::MacroBodyStart;
    }
    if token_str == "]" {
        return Token::MacroBodyEnd;
    }

    // Relative
    match &token_str[..1] {
        "&" => {
            return Token::Relative {
                offset: token_str[1..].parse::<i32>().expect("Relative"),
            };
        }
        _ => {}
    }

    // String and macro start
    match &token_str[..1] {
        "'" => {
            return Token::CharLiteral {
                value: token_str.as_bytes()[1] as char,
            };
        }
        "\"" => {
            return Token::StrLiteral {
                value: token_str[1..token_len - 1].to_string(),
            };
        }
        "@" => {
            return Token::MacroDeclaration {
                name: token_str[1..].to_string(),
            };
        }
        "{" => return Token::Scope,
        "}" => return Token::Unscope,
        "!" => {
            return Token::MacroCall {
                name: token_str[1..].to_string(),
            };
        }
        "#" => {
            if &token_str[token_str.len()-4..] == ".sbl" {
                return Token::Namespace { name: token_str[1..token_str.len()-4].to_string() };
            }
            return Token::Namespace { name: token_str[1..].to_string() };
        }
        _ => {}
    }

    // Labels
    return Token::Label {
        name: token_str.to_string(),
    };
}

pub fn lexer(mut text: String) -> Vec<Token> {
    text = clean(text);

    let mut result_tokens: Vec<Token> = Vec::new();

    let statements = text.split('\n');
    for (line, statement) in statements.enumerate() {
        let tokens = statement.split(' ');
        for token in tokens {
            if token == "" {
                continue;
            }
            result_tokens.push(match_token(token));
        }
        result_tokens.push(Token::Linebreak);
    }
    //println!("{:?}", result_tokens);
    return result_tokens;
}
