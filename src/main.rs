use std::{fs, process::exit};


use crate::{codegen::generate, sanitiser::sanitise};
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod interpreter;
mod lexer;
mod mem_view;
mod parser;
mod sanitiser;
mod symbols;
mod tokens;
mod testing;
mod codegen;
mod preprocessor;
mod feedback;
fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug);
    //let args: Vec<String> = env::args().collect();
    // let query = &args[1];
    // let file_path = &args[2];

    //let mut mem: Vec<u16> = vec![6, 7, 3, 4, 4, 0xFFFF, 4, 5];
    // interpreter::interpret(&mut mem);
    // exit(0);


    let file_path: &str = "./InlineMacros.sbl";

    println!("In file {file_path}");
    println!("{}", log::max_level());
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    

    let mut data = assemble(contents);
    interpreter::interpret(&mut data);

    //lexer::lexer(contents);
}

fn assemble(text: String) -> Vec<u16> {
    let sanitised_text = sanitise(text);
    let mut currently_imported: Vec<String> = Vec::new();
    let with_imports = preprocessor::include_imports(sanitised_text, &mut currently_imported, true);
    
    let tokens = lexer::lexer(with_imports);
    println!("{:?}", tokens);

    let statements = parser::parse(tokens);

    return codegen::generate(statements);

}
