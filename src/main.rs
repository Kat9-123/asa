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
    terminate,
};

fn main() {
    SimpleLogger::new().init().unwrap();
    args::parse();

    log::set_max_level(args::get().feedback_level.to_log_level());
    //disable_raw_mode();
    let target = &args::get()
        .target
        .clone()
        .unwrap_or_else(|| "./Main.sbl".to_string());
    let file_path = format!("./subleq/{}", target);
    println!("Assembling {file_path}");
    let contents = fs::read_to_string(&file_path);
    let contents = contents.unwrap_or_else(|e| {
        log::error!("Error reading file: {file_path}. {e}");
        terminate!();
    });

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

    file.write_all(&to_bin(&mem)).unwrap();
    if args::get().debugger {
        debugger::run_with_debugger(&mut mem, &tokens, false);
    } else {
        let result = interpreter::interpret_fast(&mut mem);
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
