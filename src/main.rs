use simple_logger::SimpleLogger;
use std::{
    fs::{self},
    time::Instant,
};

use asa::{
    args::{self},
    assembler,
    feedback::asm_runtime_error,
    files::{self, OutputFile},
    runtimes::debugger,
    runtimes::interpreter,
    terminate, utils,
};

macro_rules! println_silenceable {
    ($($arg:tt)*) => {
        if !args::get().silent {
            println!($($arg)*);
        }
    };
}

fn main() {
    // Setup
    SimpleLogger::new().init().unwrap();
    args::parse();
    log::set_max_level(args::get().feedback_level.to_log_level());
    if args::get().silent {
        log::set_max_level(log::LevelFilter::Error);
    }

    let (target, module) = files::get_target_and_module_name(args::get().target.clone());
    let output_file = OutputFile::new(&args::get().output, module.clone());

    // Assembly
    println_silenceable!("Assembling {target:?}, {module}");
    let contents = fs::read_to_string(&target);
    let contents = contents.unwrap_or_else(|e| {
        log::error!("Error reading file: {target:?}. {e}");
        terminate!();
    });

    let timer = Instant::now();
    let (mut mem, tokens) = assembler::assemble(&contents, target.to_str().unwrap().to_string());
    println_silenceable!("\nAssembled in: {:.3?}", timer.elapsed());
    println_silenceable!(
        "Size: {}/{}, {:.4}%",
        mem.len(),
        0xFFFF,
        (mem.len() as f32 / 0xFFFF as f32) * 100f32
    );
    files::to_file(&mem, output_file);

    if args::get().disable_execution {
        return;
    }

    // Execution
    println_silenceable!("Running...");

    println_silenceable!("{}", "-".repeat(80));

    if args::get().debugger {
        debugger::run_with_debugger(&mut mem, &tokens, false);
        return;
    }

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
