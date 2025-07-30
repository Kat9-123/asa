use std::{fs, path::PathBuf, process::exit};

use crate::{feedback::terminate, tokens::Token};
use log::{LevelFilter, debug, info};
use simple_logger::SimpleLogger;
use std::env;

use asa::*;
fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Info);
    args::read();
    //disable_raw_mode();
    let file_path = format!("./subleq/{}", args::get().target);

    info!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path);
    let contents = match contents {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error reading file: {}. {}", file_path, e);
            exit(1)
        }
    };

    let (mut mem, tokens) = assembler::assemble(&contents, file_path);
    //  println!("{:?}", mem);
    //   mem_view::draw_mem(&mem, 0);
    // debugger::debug(&mut mem, &tokens, true);
    interpreter::interpret(&mut mem, &tokens, false);
    //  interpreter::interpret_fast(&mut mem);
}
