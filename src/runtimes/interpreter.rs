use std::{
    num::Wrapping,
    time::{Duration, Instant},
};

use crossterm::event::KeyCode;

use crate::runtimes::debugger::get_key;

const IO_ADDR: i16 = -1;
const DEBUG_ADDR: i16 = -2;
const PERF_ADDR: i16 = -3;

/*
pub fn die(instruction_logs: &Vec<InstructionLog>, tokens: &Vec<Token>, pc: usize, reason: (usize, &str), first: (usize, &str), second: (usize, &str)) {
    trace(instruction_logs, tokens);

    asm_error_no_terminate!(&tokens[pc + reason.0].info, reason.1);
    asm_details!(&tokens[pc + 1].info, "'B' part");
    asm_details!(&tokens[pc + 2].info, "'C' part");
    terminate();
}
    */
#[derive(Debug, PartialEq, Eq, Clone)]

pub struct InstructionHistoryItem {
    pub pc: usize,
    pub original_value_at_b: u16,
    pub io_operation: IOOperation,
}

#[derive(Debug)]
pub enum RuntimeError {
    COutOfRange(usize),
    AOutOfRange(usize),
    BOutOfRange(usize),
    Breakpoint(usize),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IOOperation {
    Char(char),
    Debug(u16),
    Halt,
    BreakPoint,
    Perf,
    None,
}

pub fn interpret(mem: &mut [u16]) -> (Result<String, RuntimeError>, u128, Duration) {
    let mut prev_pc: usize = 0xFFFF;
    let mut pc = 0;
    let mut total_ran: u128 = 0;
    let mut io_buffer = String::new();
    let mut io_time: Duration = Duration::new(0, 0);
    loop {
        total_ran += 1;
        if pc + 2 >= mem.len() {
            return (Err(RuntimeError::COutOfRange(prev_pc)), total_ran, io_time);
        }
        let a = mem[pc] as usize;
        let b = mem[pc + 1] as usize;
        let c = mem[pc + 2] as usize;

        let mut result: u16 = 0;
        if b == 0xFFFF {
            result = mem[a];
            let ch = result as u8 as char;
            io_buffer.push(ch);

            let timer = Instant::now();
            print!("{ch}");
            io_time += timer.elapsed();
        } else if b == 0xFFFE {
            result = mem[a];
            let ch = (result as i16).to_string();
            io_buffer.push_str(&ch);

            let timer = Instant::now();
            println!("{ch}");
            io_time += timer.elapsed();
        } else if a == 0xFFFF {
            let timer = Instant::now();

            let c = match get_key() {
                KeyCode::Char(x) => x,
                _ => '\0',
            };
            io_time += timer.elapsed();
            mem[b] = c as u16;
        } else {
            if a >= mem.len() {
                return (Err(RuntimeError::AOutOfRange(pc)), total_ran, io_time);
            }
            if b >= mem.len() {
                return (Err(RuntimeError::BOutOfRange(pc)), total_ran, io_time);
            }
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }
        prev_pc = pc;
        if result as i16 <= 0 {
            //  println!("JUMP!");
            if c as i16 == IO_ADDR {
                break;
            }
            match c as i16 {
                IO_ADDR => break,
                DEBUG_ADDR => pc += 3,
                _ => pc = c,
            }
        } else {
            pc += 3;
        }
    }
    (Ok(io_buffer), total_ran, io_time)
}
