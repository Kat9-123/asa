use std::{fs::File, io::Write, path::Path, process::Output};

use crate::{
    args,
    tokens::{Token, TokenVariant},
};

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
