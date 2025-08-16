use simple_logger::SimpleLogger;
use std::{
    fs::{self, File},
    io::Write,
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

fn with_thousands(s: String) -> String {
    s.as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

fn main() {
    SimpleLogger::new().init().unwrap();
    args::parse();

    log::set_max_level(args::get().feedback_level.to_log_level());
    //disable_raw_mode();
    let target = &args::get()
        .target
        .clone()
        .unwrap_or_else(|| "./Main.sbl".to_string());
    let file_path = format!("./subleq/{target}");
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

    println!("{}", "-".repeat(80));
    let mut file = File::create("test.bin").unwrap();
    let timer = Instant::now();

    file.write_all(&to_bin(&mem)).unwrap();

    let (result, total_ran, io_time) = interpreter::interpret_fast(&mut mem);
    let elapsed = timer.elapsed();
    let compute_time = elapsed - io_time;
    println!("{}\n", "-".repeat(80));
    if let Err(e) = result {
        asm_runtime_error(e, &tokens)
    }
    println!("Execution took: {elapsed:.3?}");
    println!("Time spent on IO: {io_time:.3?}");
    println!("Time spent on instructions: {compute_time:.3?}\n");
    println!(
        "Instruction executed: {}",
        with_thousands(total_ran.to_string())
    );
    println!(
        "Instructions per second: {}",
        with_thousands(((total_ran as f64 / compute_time.as_secs_f64()) as u128).to_string())
    );
    //if let Err(e) = result {
    //    asm_runtime_error(e, &tokens);
    //}
    //interpreter::interpret_fast(&mut mem);
}
