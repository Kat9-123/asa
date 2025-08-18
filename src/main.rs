use simple_logger::SimpleLogger;
use std::{
    fs::{self},
    path::PathBuf,
    time::Instant,
};

use asa::{
    args::{self},
    assembler,
    feedback::asm_runtime_error,
    files::{self, InputFileType, OutputFile, from_bin, from_text},
    runtimes::{debugger, interpreter},
    terminate,
    tokens::Token,
    utils,
};

macro_rules! println_silenceable {
    ($($arg:tt)*) => {
        if !args::get().silent {
            println!($($arg)*);
        }
    };
}

fn process_input_file(
    target: &PathBuf,
    input_file_type: InputFileType,
) -> (Vec<u16>, Option<Vec<Token>>) {
    match input_file_type {
        InputFileType::Sublang => {
            let contents = fs::read_to_string(&target);
            let contents = contents.unwrap_or_else(|e| {
                log::error!("Error reading file: {target:?}. {e}");
                terminate!();
            });
            println_silenceable!("Assembling {target:?}");
            let timer = Instant::now();
            let (mem, tokens) =
                assembler::assemble(&contents, target.to_str().unwrap().to_string());
            let tokens = Some(tokens);
            println_silenceable!("\nAssembled in: {:.3?}", timer.elapsed());
            println_silenceable!(
                "Size: {}/{}, {:.4}%",
                mem.len(),
                0xFFFF,
                (mem.len() as f32 / 0xFFFF as f32) * 100f32
            );

            (mem, tokens)
        }
        InputFileType::Binary => {
            let contents = fs::read(&target);
            let contents = contents.unwrap_or_else(|e| {
                log::error!("Error reading file: {target:?}. {e}");
                terminate!();
            });
            (from_bin(&contents), None)
        }
        InputFileType::Plaintext => {
            let contents = fs::read_to_string(&target);
            let contents = contents.unwrap_or_else(|e| {
                log::error!("Error reading file: {target:?}. {e}");
                terminate!();
            });
            (from_text(&contents), None)
        }
    }
}

fn main() {
    // Setup
    SimpleLogger::new().init().unwrap();
    args::parse();
    log::set_max_level(args::get().feedback_level.to_log_level());
    if args::get().silent {
        log::set_max_level(log::LevelFilter::Error);
    }

    let (target, input_file_type, module) =
        files::get_target_and_module_name(args::get().target.clone());
    let output_file = OutputFile::new(&args::get().output, module.clone());

    // Assembly
    let (mut mem, tokens) = process_input_file(&target, input_file_type);
    files::to_file(&mem, output_file);

    if args::get().disable_execution {
        return;
    }

    // Execution
    println_silenceable!("Running...");

    println_silenceable!("{}", "-".repeat(80));

    if args::get().debugger {
        if let Some(tokens) = tokens {
            debugger::run_with_debugger(&mut mem, &tokens, false);
        } else {
            log::error!("Can't run an SBLX or BIN file with debugger");
        }
        return;
    }
    let timer = Instant::now();
    let (result, total_ran, io_time) = interpreter::interpret(&mut mem);
    let elapsed = timer.elapsed();
    let compute_time = elapsed - io_time;
    println_silenceable!("{}", "-".repeat(80));
    if let Err(e) = result {
        asm_runtime_error(e, &tokens)
    }
    println_silenceable!("\nExecution took: {elapsed:.3?}");
    println_silenceable!("Time spent on IO: {io_time:.3?}");
    println_silenceable!("Time spent on instructions: {compute_time:.3?}\n");
    println_silenceable!(
        "Instructions executed: {}",
        utils::with_thousands(total_ran.to_string())
    );
    println_silenceable!(
        "Instructions per second: {}",
        utils::with_thousands(
            ((total_ran as f64 / compute_time.as_secs_f64()) as u128).to_string()
        )
    );
}
