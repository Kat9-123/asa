use std::{
    io::{self, Write},
    num::Wrapping,
};

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
    InstructionOutOfRange(usize),
    AOutOfRange(usize),
    BOutOfRange(usize),
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
    mem: &mut [u16],
    pc: usize,
) -> Result<(usize, IOOperation, InstructionHistoryItem), RuntimeError> {
    let a = if pc < mem.len() {
        mem[pc] as usize
    } else {
        return Err(RuntimeError::InstructionOutOfRange(pc));
    };
    let b = if pc + 1 < mem.len() {
        mem[pc + 1] as usize
    } else {
        return Err(RuntimeError::InstructionOutOfRange(pc));
    };
    let c = if pc + 2 < mem.len() {
        mem[pc + 2] as usize
    } else {
        return Err(RuntimeError::InstructionOutOfRange(pc));
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
            return Err(RuntimeError::AOutOfRange(pc));
        }
        if b >= mem.len() {
            return Err(RuntimeError::BOutOfRange(pc));
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
                        io_operation: io,
                    },
                ));
            }
        }
    }
    Ok((
        pc + 3,
        io.clone(),
        InstructionHistoryItem {
            pc,
            original_value_at_b,
            io_operation: io,
        },
    ))
}

pub fn interpret(mem: &mut [u16], return_output: bool) -> Result<Option<String>, RuntimeError> {
    let mut pc: usize = 0;
    let mut buf = String::new();
    loop {
        let (new_pc, io_operation, _) = interpret_single(mem, pc)?;
        pc = new_pc;
        match io_operation {
            IOOperation::Char(ch) => {
                buf.push(ch);
                print!("{ch}");
                let _ = io::stdout().flush();
            }
            IOOperation::Debug(ch) => {
                let ch = ch.to_string();
                buf.push_str(&ch);
                println!("{ch}");
                let _ = io::stdout().flush();
            }
            IOOperation::Halt => {
                if return_output {
                    return Ok(Some(buf));
                }
                return Ok(None);
            }
            IOOperation::BreakPoint => {}
            IOOperation::Perf => {}
            IOOperation::None => {}
        }
    }
}

pub fn interpret_fast(mem: &mut [u16]) -> Result<(), RuntimeError> {
    let mut pc = 0;
    loop {
        // mem_view::draw_mem(&mem, pc);

        let a = if pc < mem.len() {
            mem[pc] as usize
        } else {
            return Err(RuntimeError::InstructionOutOfRange(pc));
        };
        let b = if pc + 1 < mem.len() {
            mem[pc + 1] as usize
        } else {
            return Err(RuntimeError::InstructionOutOfRange(pc));
        };
        let c = if pc + 2 < mem.len() {
            mem[pc + 2] as usize
        } else {
            return Err(RuntimeError::InstructionOutOfRange(pc));
        };

        let mut result: u16 = 0;
        if b == 0xFFFF {
            result = mem[a];
            let ch = result as u8 as char;
            //   print!("{ch}");
        } else if b == 0xFFFE {
            result = mem[a];
            // let ch = (result as i16).to_string();
            // println!("{ch}");
        } else if a == 0xFFFF {
        } else {
            if a >= mem.len() {
                return Err(RuntimeError::AOutOfRange(pc));
            }
            if b >= mem.len() {
                return Err(RuntimeError::BOutOfRange(pc));
            }
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }

        if result as i16 <= 0 {
            //  println!("JUMP!");
            if c as i16 == IO_ADDR {
                break;
            }
            pc = c;
        } else {
            pc += 3;
        }
    }

    Ok(())
}
