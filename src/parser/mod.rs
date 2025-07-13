
mod labels;
mod literals;
mod macros;
pub mod statements;

use crate::{print_debug, println_debug};
use crate::tokens::{Token, TokenVariant};
use crate::parser::labels::*;
use crate::parser::literals::*;
use crate::parser::macros::*;
use crate::parser::statements::*;

/*
fn find_and_collapse_groups(tokens: &Vec<Token>) {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    fn recursive(tokens: &Vec<Token>) -> Token {

    }
    for tok in tokens {
        match tok.variant {

        }
    }


}
 */

fn resolve_relatives(tokens: &Vec<Token>) -> Vec<Token> {
    let mut address: i32 = 0;
    let mut new_tokens: Vec<Token> = Vec::new();

    for token in tokens {
        match token.variant {
            TokenVariant::Relative {offset } => {
                new_tokens.push(Token {
                    info: token.info.clone(),
                    variant: TokenVariant::DecLiteral { value: address + offset },
                    origin_info: token.origin_info.clone()
                });
            }
            _ => new_tokens.push(token.clone())
        }
        address += token.size();
    }
    new_tokens
}



fn expand_mults(tokens: &Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if i + 1 < tokens.len() && let TokenVariant::Mult = tokens[i + 1].variant {
            match &tokens[i + 2].variant {
                TokenVariant::DecLiteral {  value: count } => {
                    for mult_i in 0..*count {
                        new_tokens.push(tokens[i].clone());
                    }
                    i += 3;
                    continue;
                }
                _ => {} // its the deref operator
            }
        }
        new_tokens.push(tokens[i].clone());
        i += 1;
    }
    new_tokens
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
        if let TokenVariant::Linebreak = token.variant {
            println_debug!("");
            continue;
        }
        print_debug!("{:?} ", token);
    }
    println_debug!();

    let tokens = convert_strings(tokens);

    let tokens = expand_mults(&tokens);
    let tokens = expand_derefs(&tokens);
    log::debug!("Derefs:");
    for token in &tokens {
        if let TokenVariant::Linebreak = token.variant {
            println_debug!("");
            continue;
        }
        print_debug!("{:?} ", token);
    }
    println_debug!();

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
    tokens
}