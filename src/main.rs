use simple_logger::SimpleLogger;
use std::{
    fs::{self, File},
    io::Write,
    process::exit,
    time::Instant,
};

use asa::{
    args, assembler,
    codegen::{from_bin, to_bin},
    interpreter,
};

fn main() {
    SimpleLogger::new().init().unwrap();
    args::read();

    log::set_max_level(args::get().feedback_level.to_log_level());
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
    // debugger::run_with_debugger(&mut mem, &tokens, true);
    let mut file = File::create("test.bin").unwrap();
    let timer = Instant::now();

    file.write_all(&to_bin(&mem));
    let _result = interpreter::interpret(&mut mem, false).unwrap();
    println!("Execution took: {:.3?}", timer.elapsed());
    //if let Err(e) = result {
    //    asm_runtime_error(e, &tokens);
    //}
    //interpreter::interpret_fast(&mut mem);
}
