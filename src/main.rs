use simple_logger::SimpleLogger;
use std::{
    fs::{self, File},
    io::Write,
    process::exit,
    time::Instant,
};

use asa::{
    args, assembler,
    codegen::to_bin,
    debugger,
    feedback::asm_runtime_error,
    interpreter::{self},
};

fn main() {
    SimpleLogger::new().init().unwrap();
    args::parse();

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
    println!("\nAssembled in: {:.3?}", timer.elapsed());
    println!(
        "Size: {}/{}, {:.4}%",
        mem.len(),
        0xFFFF,
        (mem.len() as f32 / 0xFFFF as f32) * 100f32
    );
    println!("Running...");

    let mut file = File::create("test.bin").unwrap();
    let timer = Instant::now();

    file.write_all(&to_bin(&mem));
    if args::get().debugger {
        debugger::run_with_debugger(&mut mem, &tokens, false);
    } else {
        let result = interpreter::interpret(&mut mem, false);
        match result {
            Err(e) => asm_runtime_error(e, &tokens),
            _ => {}
        }
    }
    println!("\nExecution took: {:.3?}\n", timer.elapsed());
    //if let Err(e) = result {
    //    asm_runtime_error(e, &tokens);
    //}
    //interpreter::interpret_fast(&mut mem);
}
