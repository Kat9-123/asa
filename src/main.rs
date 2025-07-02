use std::{fs, process::exit};

use crate::{codegen::generate, sanitiser::sanitise};
use simple_logger::SimpleLogger;

mod interpreter;
mod lexer;
mod mem_view;
mod parser;
mod sanitiser;
mod scopes;
mod symbols;
mod tokens;
mod codegen;

fn main() {
    SimpleLogger::new().init().unwrap();
    //let args: Vec<String> = env::args().collect();

    // let query = &args[1];
    // let file_path = &args[2];

    //let mut mem: Vec<u16> = vec![6, 7, 3, 4, 4, 0xFFFF, 4, 5];
    // interpreter::interpret(&mut mem);
    // exit(0);
    let file_path: &str = "Scopes.sbl";

    println!("In file {file_path}");

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    let sanitised_text = sanitise(contents);
    let tokens = lexer::lexer(sanitised_text);
    println!("{:?}", tokens);
    let statements = parser::parse(tokens);
    generate(statements);
    //lexer::lexer(contents);
}
