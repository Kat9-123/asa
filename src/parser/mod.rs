
mod labels;
mod literals;
mod macros;
pub mod statements;

use crate::asm_error;
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


fn add_relatives(tokens: &Vec<Token>) {
    let mut new_tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {

        // Either B -= A or B -= X -> A
        if let Token::Subleq { .. } = tokens[i] {
            if let Token::Linebreak { .. } = tokens[i+2] {
                new_tokens.push(tokens[i].clone())
            }
        }
    }
}




pub fn parse(tokens: Vec<Token>) -> Vec<Token> {

    

    let (mut tokens, macros) = read_macros(tokens);

    log::debug!("Found macros:");
    for i in &macros {
        println!("{i:?}");

    }
    println!();


    tokens = loop_insert_macros(tokens, &macros);

    log::debug!("Inserted macros:");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();


    let mut tokens = expand_strings(tokens);
    char_and_hex_to_dec(&mut tokens);
    
    log::debug!("Converted literals:");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();


    let tokens = grab_braced_label_definitions(tokens);

    let tokens = separate_statements(&tokens);

    log::debug!("Statements");
    for statement in &tokens {
        println!("{:?}", statement);
    }
    println!();


    let scoped_label_table = assign_addresses_to_labels(&tokens);

    log::debug!("Label Table");
    println!("{:?}", scoped_label_table);
    println!();

    log::debug!("Label Table");

    //   let label_table: HashMap<String, i32> = assign_addresses_to_labels(&statements);
    let tokens = resolve_labels(&tokens, &scoped_label_table);
    for statement in &tokens {
        println!("{:?}", statement);
    }
    println!();



    let tokens = resolve_relatives(&tokens);
    for statement in &tokens {
        println!("{:?}", statement);
    }
    return tokens;
}