mod labels;
mod literals;
mod macros;
pub mod other;

use crate::parser::labels::*;
use crate::parser::literals::*;
use crate::parser::macros::*;
use crate::parser::other::*;
use crate::tokens::{Token, TokenVariant};
use crate::{print_debug, println_debug};

pub fn parse(tokens: Vec<Token>) -> Vec<Token> {
    let mut tokens = tokens;

    char_and_hex_to_dec(&mut tokens);
    let tokens = handle_assignments(&tokens);

    let tokens = grab_braced_label_definitions(tokens);
    let (mut tokens, macros) = read_macros(tokens);
    log::debug!("Found macros:");
    for i in &macros {
        println_debug!("{}", i.1);
    }
    println_debug!();
    //println!("{}", tokens.len());

    tokens = insert_macros(tokens, &macros, vec![]);
    //println!("{}", tokens.len());
    //let  tokens = loop_insert_macros(tokens, &macros);

    log::debug!("Inserted macros:");
    for token in &tokens {
        if let TokenVariant::Linebreak = token.variant {
            println_debug!("");
            continue;
        }
        print_debug!("{:?}  ", token);
    }
    println_debug!();

    let tokens = convert_strings(tokens);
    let tokens = expand_mults(&tokens);
    let tokens = expand_derefs(&tokens);

    log::debug!("Derefs and Literals");
    for token in &tokens {
        if let TokenVariant::Linebreak = token.variant {
            println_debug!("");
            continue;
        }
        print_debug!("{:?}  ", token);
    }
    println_debug!();

    let mut tokens = fix_instructions_and_collapse_label_definitions(&tokens);

    log::debug!("Fixed");
    for statement in &tokens {
        println_debug!("{:?}", statement);
    }
    println_debug!();

    let scoped_label_table = assign_addresses_to_labels(&tokens);

    log::debug!("Label Table");

    println_debug!("{:?}", scoped_label_table);
    println_debug!();

    resolve_labels_and_relatives(&mut tokens, &scoped_label_table);
    log::debug!("Resolved Labels");

    for statement in &tokens {
        println_debug!("{:?}", statement);
    }

    tokens
}
