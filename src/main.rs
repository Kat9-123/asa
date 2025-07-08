use std::{fs, path::PathBuf, process::exit};


use crate::{codegen::generate, sanitiser::sanitise, tokens::Token};
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod parser;


mod interpreter;
mod mem_view;
mod sanitiser;
mod symbols;
mod tokens;
mod testing;
mod codegen;
mod preprocessor;
mod feedback;
mod new_lexer;
fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug);
    //let args: Vec<String> = env::args().collect();
    // let query = &args[1];
    // let file_path = &args[2];

    //let mut mem: Vec<u16> = vec![6, 7, 3, 4, 4, 0xFFFF, 4, 5];
    // interpreter::interpret(&mut mem);
    // exit(0);


    let file_path: &str = "./subleq/STDTest.sbl";

    println!("In file {file_path}");
    println!("{}", log::max_level());
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    

    let mut data = assemble(contents);
    interpreter::interpret(&mut data);

    //lexer::lexer(contents);
}

fn assemble(text: String) -> Vec<u16> {
   // let sanitised_text = sanitise(text);
    let mut currently_imported: Vec<PathBuf> = Vec::new();

    let cleaned_string: String = text.replace("\r\n", "\n").replace("\t", " ");

    let start_line_number = cleaned_string.matches('\n').count() as i32;

    let with_imports = preprocessor::include_imports(cleaned_string, &mut currently_imported, true);

    let delta_line_number =   (with_imports.matches('\n').count() as i32) -start_line_number - 1;
    let tokens = new_lexer::tokenise(with_imports, delta_line_number);
    /* */
    println!("TOKENS:");

    for i in &tokens {
        if let Token::Linebreak {..} = i {
            println!();
            continue;
        }
        print!("{:?}, ", i);
    }


    let statements = parser::parse(tokens);

    return codegen::generate(statements);

}
