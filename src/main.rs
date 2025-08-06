use std::{fs, process::exit};

use log::LevelFilter;
use simple_logger::SimpleLogger;

use asa::{args::FeedbackLevel, *};
use std::time::Instant;
fn main() {
    SimpleLogger::new().init().unwrap();
    args::read();

    log::set_max_level(match args::get().feedback_level {
        FeedbackLevel::Debug => LevelFilter::Debug,
        FeedbackLevel::Notes => LevelFilter::Info,
        FeedbackLevel::Warn => LevelFilter::Warn,
        FeedbackLevel::Error => LevelFilter::Error,
    });
    //disable_raw_mode();
    let file_path = format!("./subleq/{}", args::get().target);

    println!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path);
    let contents = match contents {
        Ok(c) => c,
        Err(e) => {
            log::error!("Error reading file: {file_path}. {e}");
            exit(1);
        }
    };

    let timer = Instant::now();
    let (mut mem, tokens) = assembler::assemble(&contents, file_path);
    println!("Assembled in: {:.3?}", timer.elapsed());
    println!("Running...");
    //  println!("{:?}", mem);
    //   mem_view::draw_mem(&mem, 0);
    //debugger::run_with_debugger(&mut mem, &tokens, true);
    let _result = interpreter::interpret(&mut mem, false).unwrap();
    //if let Err(e) = result {
    //    asm_runtime_error(e, &tokens);
    //}
    //interpreter::interpret_fast(&mut mem);
}
