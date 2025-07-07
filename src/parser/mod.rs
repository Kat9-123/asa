
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




fn resolve_relatives(statements: &mut Vec<Statement>) {
    let mut address: i32 = 0;

    for statement in statements {
        match statement {
            Statement::Instruction { a, b, c } => {
                if let Token::Relative { offset } = a {
                    *a = Token::DecLiteral {
                        value: address + *offset,
                    }
                }
                if let Token::Relative { offset } = b {
                    *b = Token::DecLiteral {
                        value: address + 1 + *offset,
                    }
                }
                if let Token::Relative { offset } = c {
                    *c = Token::DecLiteral {
                        value: address + 2 + *offset,
                    }
                }
            },
            Statement::Literal { x } => {
                if let Token::Relative { offset } = x {
                    *x = Token::DecLiteral {
                        value: address + *offset,
                    }
                }
            },

            _ => {}

        }
        address += statement.size();
    }
}





pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {

    

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




    let mut statements = separate_statements(&tokens);

    log::debug!("Statements");
    for statement in &statements {
        println!("{:?}", statement);
    }
    println!();


    let scoped_label_table = assign_addresses_to_labels(&statements);

    log::debug!("Label Table");
    println!("{:?}", scoped_label_table);
    println!();

    log::debug!("Label Table");

    //   let label_table: HashMap<String, i32> = assign_addresses_to_labels(&statements);
    resolve_labels(&mut statements, &scoped_label_table);
    for statement in &statements {
        println!("{:?}", statement);
    }
    println!();



    resolve_relatives(&mut statements);
    for statement in &statements {
        println!("{:?}", statement);
    }
    return statements;
}