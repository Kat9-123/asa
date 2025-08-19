use simple_logger::SimpleLogger;
use std::time::Instant;

use asa::{
    args::{self},
    feedback::asm_runtime_error,
    files::{self, OutputFile},
    println_silenceable,
    runtimes::{debugger, interpreter},
    utils,
};

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
    let (mut mem, tokens) = files::process_input_file(&target, input_file_type);
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
            log::error!("Can't run an SBLX or BIN file with the debugger");
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
