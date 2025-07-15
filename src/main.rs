use std::io::Read;
use std::path::Path;
use std::{fs, path::PathBuf};

use crate::mem_view::draw_mem;
use crate::tokens::Token;
use crate::tokens::TokenVariant;
use crossterm::terminal::disable_raw_mode;
use log::{LevelFilter, debug, info};
use simple_logger::SimpleLogger;
use std::env;
mod parser;

mod codegen;
mod debugger;
mod feedback;
mod interpreter;
mod mem_view;
mod new_lexer;
mod preprocessor;
mod symbols;
mod testing;
mod tokens;
use clap::Parser;
use std::time::Instant;

/// Advanced Subleq Assembler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    target: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
    // Debug mode
    // Hide info
    // Hide warnings
    // Run with debugger
    // Type checking
    // Trace len
}

fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug);
    disable_raw_mode();

    let args: Vec<String> = env::args().collect();

    let file_path = format!("./subleq/{}", args[1]);

    info!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path).expect("Should have been able to read the file");

    let (mut mem, tokens) = assemble(contents, file_path);
    //   mem_view::draw_mem(&mem, 0);
    debugger::debug(&mut mem, &tokens, true);
    //  interpreter::interpret(&mut mem, &tokens, false, true);
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
        if let TokenVariant::Linebreak = i.variant {
            println_debug!();
            continue;
        }
        print_debug!("{:?}, ", i);
    }

    let tokens = parser::parse(tokens);
    let result = codegen::generate(tokens);
    info!("Assembled in: {:.3?}", timer.elapsed());
    println!();
    draw_mem(&result.0, 0);
    result
}
