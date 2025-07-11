
mod labels;
mod literals;
mod macros;
pub mod statements;

use crate::{asm_error, println_debug};
use crate::tokens::{self, Token};
use crate::parser::labels::*;
use crate::parser::literals::*;
use crate::parser::macros::*;
use crate::parser::statements::*;




fn resolve_relatives(tokens: &Vec<Token>) -> Vec<Token> {
    let mut address: i32 = 0;
    let mut new_tokens: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::Relative { info, offset } => {
                new_tokens.push(Token::DecLiteral { info: info.clone(), value: address + *offset })
            }
            _ => new_tokens.push(token.clone())
        }
        address += token.size();
    }
    return new_tokens;
}


fn expand_mults(tokens: &Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if i + 1 < tokens.len() && let Token::Mult {..} = tokens[i + 1] {
            match &tokens[i] {
                Token::DecLiteral { info, value: count } => {
                    for mult_i in 0..*count {
                        new_tokens.push(tokens[i + 2].clone());
                    }
                    i += 3;
                    continue;
                }
                _ => todo!(),
            }
        }
        new_tokens.push(tokens[i].clone());
        i += 1;
    }
    return new_tokens;
}


pub fn parse(tokens: Vec<Token>) -> Vec<Token> {

    let mut tokens= tokens;

    char_and_hex_to_dec(&mut tokens);

    log::debug!("Converted literals:");
    for token in &tokens {
        println_debug!("{:?}", token);
    }
    println_debug!();


    let tokens = grab_braced_label_definitions(tokens);

    let (mut tokens, macros) = read_macros(tokens);

    log::debug!("Found macros:");
    for i in &macros {
        println_debug!("{i:?}");

    }
    println_debug!();


    tokens = loop_insert_macros(tokens, &macros);

    log::debug!("Inserted macros:");
    for token in &tokens {
        println_debug!("{:?}", token);
    }
    println_debug!();

    let tokens = convert_strings(tokens);

    let tokens = expand_mults(&tokens);


    let tokens = separate_statements(&tokens);

    log::debug!("Statements");
    for statement in &tokens {
        println_debug!("{:?}", statement);
    }
    println_debug!();


    let scoped_label_table = assign_addresses_to_labels(&tokens);

    log::debug!("Label Table");
    println_debug!("{:?}", scoped_label_table);
    println_debug!();

    log::debug!("Label Table");

    let tokens = resolve_labels(&tokens, &scoped_label_table);
    for statement in &tokens {
        println_debug!("{:?}", statement);
    }
    println_debug!();



    let tokens = resolve_relatives(&tokens);
    for statement in &tokens {
        println_debug!("{:?}", statement);
    }
    return tokens;
}