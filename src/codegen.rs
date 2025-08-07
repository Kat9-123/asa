use crate::tokens::{Token, TokenVariant};

pub fn to_text(data: &Vec<u16>) -> String {
    let mut text: String = String::new();
    for i in data {
        text.push_str(&i.to_string());
        text.push(' ');
    }
    text.pop();

    text
}
pub fn generate(statements: Vec<Token>) -> (Vec<u16>, Vec<Token>) {
    let mut mem: Vec<u16> = Vec::new();
    let mut final_tokens: Vec<Token> = Vec::new();

    for statement in statements {
        match &statement.variant {
            TokenVariant::DecLiteral { value } => {
                mem.push(*value as u16);
                final_tokens.push(statement.clone());
            }
            _ => {
                continue;
            }
        }
    }
    (mem, final_tokens)
}
