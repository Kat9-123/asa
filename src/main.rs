use std::{fs, process::exit};

use log::{LevelFilter, info};
use simple_logger::SimpleLogger;

use asa::{debugger::user_input, *};
fn main() {
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug);
    args::read();
    //disable_raw_mode();
    let file_path = format!("./subleq/{}", args::get().target);

    info!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path);
    let contents = match contents {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error reading file: {}. {}", file_path, e);
            exit(1);
        }
    };

    let (mut mem, tokens) = assembler::assemble(&contents, file_path);
    //  println!("{:?}", mem);
    //   mem_view::draw_mem(&mem, 0);
   // debugger::debug(&mut mem, &tokens, true, user_input);
    interpreter::interpret(&mut mem, false).unwrap();
    //  interpreter::interpret_fast(&mut mem);
}
