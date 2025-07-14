use std::{
    io::{self, Write},
    num::Wrapping,
};

use log::info;

use crate::{
    asm_details, asm_error_no_terminate, asm_instruction, asm_sub_instruction, feedback::terminate, mem_view, tokens::Token
};

use crossterm::{ExecutableCommand, cursor, terminal};

const IO_ADDR: i16 = -1;
const DEBUG_ADDR: i16 = -2;
const PERF_ADDR: i16 = -3;

fn outside_mem_bounds_err(tokens: &Vec<Token>, prev_pc: usize) {
    asm_error_no_terminate!(&tokens[prev_pc + 2].info, "Jump outside of memory bounds");
    asm_details!(&tokens[prev_pc].info, "'A' part");
    asm_details!(&tokens[prev_pc + 1].info, "'B' part");
    terminate();
}

struct InstructionLog {
    pub pc: usize,
    pub jumped: bool,
}

fn instruction_info(tokens: &Vec<Token>, jumped: bool, pc: usize) {
    let mut stdout = io::stdout();
    //  stdout.execute(terminal::Clear(terminal::ClearType::All));
    //  stdout.execute(cursor::MoveTo(0,0));
    if let Some(x) = &tokens[pc].origin_info {
        asm_instruction!(x, "Instruction");
    } else {
        asm_instruction!(&tokens[pc].info, "'Instruction");
    }

    asm_sub_instruction!(&tokens[pc].info, "'A' part");

    asm_sub_instruction!(&tokens[pc + 1].info, "'B' part");
    if jumped {
        asm_sub_instruction!(&tokens[pc + 2].info, "'C' part. JUMPED");
    } else {
        asm_sub_instruction!(&tokens[pc + 2].info, "'C' part. DIDN'T JUMP");
    }
    println!();
}

fn trace(instruction_logs: &Vec<InstructionLog>, tokens: &Vec<Token>) {
    for i in instruction_logs {
        instruction_info(tokens, i.jumped, i.pc)
    }
}

/*
pub fn die(instruction_logs: &Vec<InstructionLog>, tokens: &Vec<Token>, pc: usize, reason: (usize, &str), first: (usize, &str), second: (usize, &str)) {
    trace(instruction_logs, tokens);

    asm_error_no_terminate!(&tokens[pc + reason.0].info, reason.1);
    asm_details!(&tokens[programme_counter + 1].info, "'B' part");
    asm_details!(&tokens[programme_counter + 2].info, "'C' part");
    terminate();
}
    */
pub fn interpret(
    mem: &mut Vec<u16>,
    tokens: &Vec<Token>,
    return_output: bool,
    debugger: bool,
) -> Option<String> {
    let mut programme_counter = 0;
    let mut prev_programme_counter = 0;
    let mut buf = String::new();
    let mut instruction_logs: Vec<InstructionLog> = Vec::new();

    let mut performance_counter: Option<usize> = None;
    loop {
        // mem_view::draw_mem(&mem, programme_counter);

        let a = if programme_counter < mem.len() {
            mem[programme_counter] as usize
        } else {
            trace(&instruction_logs, tokens);
            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };
        let b = if programme_counter + 1 < mem.len() {
            mem[programme_counter + 1] as usize
        } else {
            trace(&instruction_logs, tokens);

            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };
        let c = if programme_counter + 2 < mem.len() {
            mem[programme_counter + 2] as usize
        } else {
            trace(&instruction_logs, tokens);

            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };

        let mut result: u16 = 0;

        if b == 0xFFFF {
            result = mem[a];
            let ch = result as u8 as char;
            buf.push(ch);
            print!("{ch}");
            io::stdout().flush();
        } else if b == 0xFFFE {
            result = mem[a];
            let ch = (result as i16).to_string();
            buf.push_str(&ch);
            println!("{ch}");
            io::stdout().flush();
        } else if a == 0xFFFF {
        } else {
            if a >= mem.len() {
                trace(&instruction_logs, tokens);

                asm_error_no_terminate!(&tokens[programme_counter].info, "Address A out of range");
                asm_details!(&tokens[programme_counter + 1].info, "'B' part");
                asm_details!(&tokens[programme_counter + 2].info, "'C' part");
                terminate();
            }
            if b >= mem.len() {
                trace(&instruction_logs, tokens);

                asm_error_no_terminate!(
                    &tokens[programme_counter + 1].info,
                    "Address B out of range"
                );
                asm_details!(&tokens[programme_counter].info, "'A' part");
                asm_details!(&tokens[programme_counter + 2].info, "'C' part");
                terminate();
            }
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }

        prev_programme_counter = programme_counter;
        let mut jumped = false;

        if result as i16 <= 0 {
            jumped = true;
        }

        if debugger {
            mem_view::draw_mem(&mem, programme_counter);

            instruction_info(tokens, jumped, programme_counter);
            let mut inp: String = String::new();
            io::stdin().read_line(&mut inp);
        }

        if result as i16 <= 0 {
            jumped = true;
            //  println!("JUMP!");

            match c as i16 {
                IO_ADDR => break,
                DEBUG_ADDR => {
                    // Breakpoint
                   // trace(&instruction_logs, tokens);

                    asm_error_no_terminate!(&tokens[programme_counter + 2].info, "Breakpoint");
                    let mut inp: String = String::new();
                    io::stdin().read_line(&mut inp);
                    programme_counter += 3;

                }
                PERF_ADDR => {
                    match performance_counter {
                        Some(x) => {
                            println!("{} instructions executed.", x - 1);
                            performance_counter = None;
                        }
                        None => {
                            performance_counter = Some(0);
                        }
                    }
                    programme_counter += 3;
                }

                _ => programme_counter = c,
            }
        } else {
            programme_counter += 3;
        }
        if let Some(x) = performance_counter.as_mut() {
            *x += 1;
        }

        instruction_logs.push(InstructionLog {
            pc: prev_programme_counter,
            jumped,
        });
        if instruction_logs.len() > 5 {
            instruction_logs.remove(0);
        }
    }
    if return_output {
        return Some(buf);
    }
    None
}
