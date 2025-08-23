mod labels;
mod literals;
mod macros;
mod other;

use log::LevelFilter;

use crate::parser::labels::*;
use crate::parser::literals::*;
use crate::parser::macros::*;
use crate::parser::other::*;
use crate::tokens::{Token, TokenVariant};

pub fn parse(tokens: Vec<Token>) -> Vec<Token> {
    let mut tokens = tokens;

    char_and_hex_to_dec_and_check_scopes(&mut tokens);
    let tokens = handle_assignments(&tokens);

    let tokens = grab_braced_label_definitions(tokens);
    let (mut tokens, macros) = read_macros(&tokens);

    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Found macros:");
        for i in &macros {
            println!("{:?}", i.1);
        }
        println!();
    }

    tokens = insert_macros(tokens, &macros, vec![]);
    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Inserted macros:");
        for token in &tokens {
            if let TokenVariant::Linebreak = token.variant {
                println!();
                continue;
            }
            print!("{:?}  ", token);
        }
        println!();
    }
    let tokens = convert_strings(tokens);
    let tokens = expand_mults(&tokens);
    let tokens = expand_derefs(&tokens);

    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Derefs and Literals");
        for token in &tokens {
            if let TokenVariant::Linebreak = token.variant {
                println!();
                continue;
            }
            print!("{:?}  ", token);
        }
        println!();
    }

    let mut tokens = fix_instructions_and_collapse_label_definitions(&tokens);
    // From this point forwards, memory addresses are fixed.
    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Fixed");
        for statement in &tokens {
            println!("{:?}", statement);
        }
        println!();
    }

    let scoped_label_table = assign_addresses_to_labels(&tokens);

    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Label Table");

        println!("{:?}", scoped_label_table);
        println!();
    }
    resolve_labels_and_relatives(&mut tokens, &scoped_label_table);

    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Resolved Labels");

        for statement in &tokens {
            println!("{:?}", statement);
        }
    }

    tokens
}
