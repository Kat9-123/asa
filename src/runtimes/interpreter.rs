use crate::runtimes::RuntimeError;
use crate::symbols::{DEBUG_ADDR, IO_ADDR};
use crossterm::event::KeyCode;
use std::{
    num::Wrapping,
    time::{Duration, Instant},
};

use crate::runtimes::debugger::get_key;

/// Returns the a result with either the complete program output, or a runtime error,
/// the total executions executed, and the time spent on IO
pub fn interpret(mem: &mut [u16]) -> (Result<String, RuntimeError>, u128, Duration) {
    let mut prev_pc: usize = 0xFFFF;
    let mut pc = 0;
    let mut total_ran: u128 = 0;
    let mut io_buffer = String::new();
    let mut io_time: Duration = Duration::new(0, 0);
    loop {
        if pc + 2 >= mem.len() {
            return (Err(RuntimeError::COutOfRange(prev_pc)), total_ran, io_time);
        }
        let a = mem[pc] as usize;
        let b = mem[pc + 1] as usize;
        let c = mem[pc + 2] as usize;

        let mut result: u16 = 0;

        match (a, b) {
            (a, _) if a >= mem.len() && a != IO_ADDR => {
                return (Err(RuntimeError::AOutOfRange(pc)), total_ran, io_time);
            }
            (_, b) if b >= mem.len() && b != IO_ADDR && b != DEBUG_ADDR => {
                return (Err(RuntimeError::BOutOfRange(pc)), total_ran, io_time);
            }

            // Output char
            (_, IO_ADDR) => {
                result = mem[a];
                let ch = result as u8 as char;
                io_buffer.push(ch);

                let timer = Instant::now();
                print!("{ch}");
                io_time += timer.elapsed();
            }
            // Output int
            (_, DEBUG_ADDR) => {
                result = mem[a];
                let ch = (result as i16).to_string();
                io_buffer.push_str(&ch);

                let timer = Instant::now();
                println!("{ch}");
                io_time += timer.elapsed();
            }
            // Input char
            (IO_ADDR, _) => {
                let timer = Instant::now();

                let c = match get_key() {
                    KeyCode::Char(x) => x,
                    _ => '\0',
                };
                io_time += timer.elapsed();
                mem[b] = c as u16;
            }

            (_, _) => {
                result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
                mem[b] = result;
            }
        }
        total_ran += 1;

        prev_pc = pc;
        if result as i16 <= 0 {
            match c {
                IO_ADDR => break,
                DEBUG_ADDR => {
                    return (Err(RuntimeError::Breakpoint(pc)), total_ran, io_time);
                }
                _ => pc = c,
            }
        } else {
            pc += 3;
        }
    }
    (Ok(io_buffer), total_ran, io_time)
}
