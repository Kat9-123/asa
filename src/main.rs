use std::io::Read;
use std::path::Path;
use std::rc::Rc;
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
mod lexer;
mod mem_view;
mod preprocessor;
mod symbols;
mod testing;
mod tokens;
use clap::Parser;
use std::time::Instant;


#[macro_export]
macro_rules! args {
    () => {
        ARGS.with(|a| a.clone())
    };
}
thread_local! {
    pub static ARGS: Rc<Args> = Rc::new(Args::parse());
}


/// Advanced Subleq Assembler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to assemble
    #[arg(short, long, default_value = "")]
    pub target: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    pub count: u8,
    // Debug mode
    //#[arg(long, default_value_t = false)]
    // assembler_debug_mode: bool,
    // Hide info
    // Hide warnings
    // Run with debugger
    /// Root path
    // #[arg(long)]
    // root_path: String,
    /// Out file
   // #[arg(short, long,default_value_t = "")]
   // out_file: str,
    /// Disable type checking
    #[arg(long, default_value_t = false)]
    pub disable_type_checking: bool,
    /// Disable notes
    #[arg(long, default_value_t = false)]
    pub disable_notes: bool,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);
    //disable_raw_mode();
    let args = Args::parse();
    // let args: Vec<String> = env::args().collect();
        let file_path = format!("./subleq/{}", args.target);

    info!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path).expect("Should have been able to read the file");

    let (mut mem, tokens) = assemble(contents, file_path);
    println!("{:?}", mem);
    //   mem_view::draw_mem(&mem, 0);
    //debugger::debug(&mut mem, &tokens, true);
    interpreter::interpret(&mut mem, &tokens, false);
}

fn assemble(text: String, path: String) -> (Vec<u16>, Vec<Token>) {
    let timer = Instant::now();

    let mut currently_imported: Vec<PathBuf> = vec![Path::new(&path).to_path_buf()];

    let with_imports = preprocessor::include_imports(text, &mut currently_imported);
    debug!("With imports: ");
    print_debug!("{}", with_imports);

    let tokens = lexer::tokenise(with_imports, path);
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
