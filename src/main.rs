use std::path::Path;
use std::{fs, path::PathBuf, process::exit};


use crate::{codegen::generate, sanitiser::sanitise, tokens::Token};
use log::{trace, LevelFilter};
use simple_logger::SimpleLogger;
use std::env;
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
use std::time::Instant;


fn main() {

    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);
    let args: Vec<String> = env::args().collect();


    //let args: Vec<String> = env::args().collect();
    // let query = &args[1];
    // let file_path = &args[2];

    //let mut mem: Vec<u16> = vec![6, 7, 3, 4, 4, 0xFFFF, 4, 5];
    // interpreter::interpret(&mut mem);
    // exit(0);
    let file_path = format!("./subleq/{}", args[1]);

    info!("{file_path}");
    let contents = fs::read_to_string(&file_path).expect("Should have been able to read the file");

    let (mut mem, tokens) = assemble(contents, file_path);
    interpreter::interpret(&mut mem, &tokens);

    //lexer::lexer(contents);
}

fn assemble(text: String, path: String) -> (Vec<u16>, Vec<Token>) {
    let before = Instant::now();

   // let sanitised_text = sanitise(text);
    let mut currently_imported: Vec<PathBuf> = vec![Path::new(&path).to_path_buf()];

    let cleaned_string: String = text.replace("\r\n", "\n").replace("\t", " ");


    let with_imports = preprocessor::include_imports(cleaned_string, &mut currently_imported, true);

    let tokens = new_lexer::tokenise(with_imports, path);
    /* */
    println_debug!("TOKENS:");

    for i in &tokens {
        if let Token::Linebreak {..} = i {
            println_debug!();
            continue;
        }
        print_debug!("{:?}, ", i);
    }


    let statements = parser::parse(tokens);
    let result = codegen::generate(statements);
    info!("Assembled in: {:.3?}", before.elapsed());
    return result;

}
