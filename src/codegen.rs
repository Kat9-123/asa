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

pub fn to_bin(data: &[u16]) -> Vec<u8> {
    let mut u8data: Vec<u8> = Vec::with_capacity(data.len() * 2);

    for i in data {
        u8data.push((i >> 8) as u8);
        u8data.push((i & 0xFF) as u8);
    }

    u8data
}

pub fn from_bin(data: &[u8]) -> Vec<u16> {
    let mut u16data: Vec<u16> = Vec::with_capacity((data.len() / 2) + 1); // Is +1 necessary

    for i in (0..data.len()).step_by(2) {
        u16data.push(((data[i] as u16) << 8) + (data[i + 1] as u16));
    }

    u16data
}
pub fn generate(statements: Vec<Token>) -> (Vec<u16>, Vec<Token>) {
    let mut mem: Vec<u16> = Vec::with_capacity(statements.len());
    let mut final_tokens: Vec<Token> = Vec::with_capacity(statements.len());
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
