use std::num::Wrapping;
use std::path::Path;
use std::{fs, path::PathBuf, process::exit};


use crate::{codegen::generate, tokens::Token};
use log::{debug, info, trace, LevelFilter};
use simple_logger::SimpleLogger;
use std::env;
mod parser;


mod interpreter;
mod mem_view;
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

    let file_path = format!("./subleq/{}", args[1]);

    info!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path).expect("Should have been able to read the file");

    let (mut mem, tokens) = assemble(contents, file_path);
    interpreter::interpret(&mut mem, &tokens, false);

}

fn assemble(text: String, path: String) -> (Vec<u16>, Vec<Token>) {
    let timer = Instant::now();

    let mut currently_imported: Vec<PathBuf> = vec![Path::new(&path).to_path_buf()];



    let with_imports = preprocessor::include_imports(text, &mut currently_imported, true);
    debug!("With imports: ");
    print_debug!("{}", with_imports);

    let tokens = new_lexer::tokenise(with_imports, path);
    println_debug!("Tokens:");

    for i in &tokens {
        if let Token::Linebreak {..} = i {
            println_debug!();
            continue;
        }
        print_debug!("{:?}, ", i);
    }


    let tokens = parser::parse(tokens);
    let result = codegen::generate(tokens);
    info!("Assembled in: {:.3?}", timer.elapsed());
    return result;

}
