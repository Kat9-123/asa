use std::{
    io::{self, Write},
    num::Wrapping,
};

use log::info;

use crate::{
    asm_details, asm_error_no_terminate, asm_instruction, asm_sub_instruction, feedback::terminate,
    mem_view, tokens::Token,
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
    // let mut stdout = io::stdout();
    //  stdout.execute(terminal::Clear(terminal::ClearType::All));
    //  stdout.execute(cursor::MoveTo(0,0));
    for i in ((tokens[pc + 1].origin_info).iter()).rev() {
        asm_instruction!(&i.1, "depth Instruction {}", i.0);
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

pub struct InstructionHistoryItem {
    pub pc: usize,
    pub original_value_at_b: u16,
    pub jumped: bool,
    pub io_operation: IOOperation,
}

#[derive(Debug)]
pub enum RuntimeError {
    InstructionOutOfRange,
    AOutOfRange,
    BOutOfRange,
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

pub fn interpret_single(
    mem: &mut Vec<u16>,
    pc: usize,
) -> Result<(usize, IOOperation, InstructionHistoryItem), RuntimeError> {
    let a = if pc < mem.len() {
        mem[pc] as usize
    } else {
        return Err(RuntimeError::InstructionOutOfRange);
    };
    let b = if pc + 1 < mem.len() {
        mem[pc + 1] as usize
    } else {
        return Err(RuntimeError::InstructionOutOfRange);
    };
    let c = if pc + 2 < mem.len() {
        mem[pc + 2] as usize
    } else {
        return Err(RuntimeError::InstructionOutOfRange);
    };

    let mut original_value_at_b = 0;

    let mut result: u16 = 0;
    let mut io: IOOperation = IOOperation::None;
    if b == 0xFFFF {
        result = mem[a];
        io = IOOperation::Char(result as u8 as char);
    } else if b == 0xFFFE {
        result = mem[a];
        io = IOOperation::Debug(result);
    } else if a == 0xFFFF {
    } else {
        if a >= mem.len() {
            return Err(RuntimeError::AOutOfRange);
        }
        if b >= mem.len() {
            return Err(RuntimeError::BOutOfRange);
        }
        original_value_at_b = mem[b];
        result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
        mem[b] = result;
    }

    if result as i16 <= 0 {
        //  println!("JUMP!");

        match c as i16 {
            IO_ADDR => {
                return Ok((
                    pc,
                    IOOperation::Halt,
                    InstructionHistoryItem {
                        pc,
                        original_value_at_b,
                        jumped: true,
                        io_operation: IOOperation::Halt,
                    },
                ));
            }
            DEBUG_ADDR => {
                // Breakpoint

                return Ok((
                    pc + 3,
                    IOOperation::BreakPoint,
                    InstructionHistoryItem {
                        pc,
                        original_value_at_b,
                        jumped: true,
                        io_operation: IOOperation::BreakPoint,
                    },
                ));
            }
            PERF_ADDR => {
                todo!();
            }

            _ => {
                return Ok((
                    c,
                    io.clone(),
                    InstructionHistoryItem {
                        pc,
                        original_value_at_b,
                        jumped: true,
                        io_operation: io,
                    },
                ));
            }
        }
    }
    return Ok((
        pc + 3,
        io.clone(),
        InstructionHistoryItem {
            pc,
            original_value_at_b,
            jumped: false,
            io_operation: io,
        },
    ));
}

pub fn interpret(
    mem: &mut Vec<u16>,
    tokens: &Vec<Token>,
    return_output: bool,
) -> Result<Option<String>, RuntimeError> {
    let mut programme_counter = 0;
    let mut prev_programme_counter = 0;
    let mut buf = String::new();

    let mut performance_counter: Option<usize> = None;
    loop {
        // mem_view::draw_mem(&mem, programme_counter);

        let a = if programme_counter < mem.len() {
            mem[programme_counter] as usize
        } else {
            return Err(RuntimeError::InstructionOutOfRange);

        };
        let b = if programme_counter + 1 < mem.len() {
            mem[programme_counter + 1] as usize
        } else {
            return Err(RuntimeError::InstructionOutOfRange);

        };
        let c = if programme_counter + 2 < mem.len() {
            mem[programme_counter + 2] as usize
        } else {
            return Err(RuntimeError::InstructionOutOfRange);
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
                return Err(RuntimeError::AOutOfRange);

            }
            if b >= mem.len() {
                return Err(RuntimeError::BOutOfRange);


            }
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }

        prev_programme_counter = programme_counter;
        let mut jumped = false;

        if result as i16 <= 0 {
            jumped = true;
        }


        if result as i16 <= 0 {
            jumped = true;
            //  println!("JUMP!");

            match c as i16 {
                IO_ADDR => break,
                DEBUG_ADDR => {
                    // Breakpoint
                    // trace(&instruction_logs, tokens);

                    programme_counter += 3;
                }
                PERF_ADDR => {
                    /*
                    match performance_counter {
                        Some(x) => {
                            println!("{} instructions executed.", x - 1);
                            performance_counter = None;
                        }
                        None => {
                            performance_counter = Some(0);
                        }
                    }
                     */
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

    }
    if return_output {
        return Ok(Some(buf));
    }
    Ok(None)
}
