use std::{fs, path::PathBuf};

use log::{LevelFilter, debug, info};
use simple_logger::SimpleLogger;
use std::env;
mod parser;

mod args;
mod assembler;
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
fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug);
    args::read();
    //disable_raw_mode();
    let file_path = format!("./subleq/{}", args::get().target);

    info!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path).expect("Should have been able to read the file");

    let (mut mem, tokens) = assembler::assemble(contents, file_path);
    println!("{:?}", mem);
    //   mem_view::draw_mem(&mem, 0);
    //debugger::debug(&mut mem, &tokens, true);
    interpreter::interpret(&mut mem, &tokens, false);
}
