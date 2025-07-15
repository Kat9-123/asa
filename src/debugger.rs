use crate::{asm_details, asm_error_no_terminate, asm_instruction};
use crate::feedback::terminate;
use crate::interpreter::RuntimeError;
use crate::{
    interpreter::{self, IOOperation, InstructionHistoryItem},
    mem_view,
    tokens::{Info, Token},
};
use colored::Colorize;
use crossterm::{
    ExecutableCommand,
    event::{Event, KeyCode, KeyEventKind, read},
    terminal::{self, enable_raw_mode},
};
use std::fmt::LowerExp;
use std::{
    io::{self, *},
    num::Wrapping,
};
use std::{fs, thread, time};

pub fn revert_historic_instruction(
    mem: &mut Vec<u16>,
    inst: &InstructionHistoryItem,
    io_buffer: &mut String,
) -> usize {
    let b = mem[inst.pc + 1] as usize;
    // let c = mem[inst.pc + 2] as usize;
    if b < mem.len() {
        mem[b] = inst.original_value_at_b;
    }

    if let IOOperation::Char(c) = inst.io_operation {
        io_buffer.pop();
    }
    return inst.pc;
    // if inst.jumped {
    //     return mem[c] as usize;
    // }
    // return inst.pc + 3;
}

pub fn error(e: RuntimeError, pc: usize, tokens: &Vec<Token>) {
    match e {
        RuntimeError::InstructionOutOfRange => {
            asm_error_no_terminate!(&tokens[pc + 2].info, "Jump outside of memory bounds");
            asm_details!(&tokens[pc].info, "'A' part");
            asm_details!(&tokens[pc + 1].info, "'B' part");
        },
        _ => {}
    }
}

pub fn apply_historic_instruction(
    mem: &mut Vec<u16>,
    inst: &InstructionHistoryItem,
    io_buffer: &mut String,
) -> usize {
    let a = mem[inst.pc] as usize;

    let b = mem[inst.pc + 1] as usize;
    let c = mem[inst.pc + 2] as usize;
    println!("{:?}", inst.io_operation);
    if let IOOperation::Char(c) = inst.io_operation {
        io_buffer.push(c);
    }
    if a < mem.len() && b < mem.len() {
        mem[b] = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
    }
    // let c = mem[inst.pc + 2] as usize;

    if inst.jumped && c <= mem.len() {
        return c as usize;
    }
    return inst.pc + 3;
}

fn address_to_string(addr: u16, mem: &Vec<u16>) -> String{
    match addr {
        0xFFFF => {
            "IO".to_string()
        },
        0xFFFE => {
            "Debug".to_string()
        }
        0xFFFD => {
            "Perf".to_string()
        },
        _ => "Perf".to_string() //format!("{}", mem[addr as usize] as i16)
    }
}

fn get_file_contents(path: &String) -> String {
    fs::read_to_string(path).expect("Should have been able to read the file")
}
pub fn display(info: &Info, pc: usize, mem: &Vec<u16>) {
    let contents = get_file_contents(&info.file);
    let lines = contents.lines().collect::<Vec<&str>>();

    const UPPER_SIZE: i32 = 5;
    const LOWER_SIZE: i32 = 5;

    let start_line = (info.line_number - 1 - UPPER_SIZE).max(0); // 0
    let end_line = (info.line_number - 1 + LOWER_SIZE + 1).min(lines.len() as i32 - 1); // 10
    let end_line = end_line + (UPPER_SIZE + LOWER_SIZE + 1 - (end_line - start_line));
    let end_line = end_line.min(lines.len() as i32 - 1);
    for i in start_line..end_line {
        if i != info.line_number - 1{
            println!("{: >4} | {: <90}", format!("{}", i + 1).bright_cyan(), lines[i as usize]);

        } else {
            println!("{: >4} {} {: <90}", format!("{}", i + 1).bright_cyan(), ">".blue(), format!("{}", lines[i as usize]).blue());

        }
    }
    println!("PC: {:X}", pc);
    println!(" a: {: >4X}   mem[a]: {: <90}", mem[pc], address_to_string(mem[pc], mem));
    println!(" b: {: >4X}   mem[b]: {: <90}", mem[pc + 1], address_to_string(mem[pc + 1], mem));
    println!(" c: {} ", address_to_string(pc  as u16+ 2, mem));
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');
    println!("{: <100}", ' ');

}


pub fn debug(mem: &mut Vec<u16>, tokens: &Vec<Token>, mut in_debugging_mode: bool) {
    let mut pc = 0;
    let mut current_instruction_idx = 0;
    let mut instruction_history: Vec<InstructionHistoryItem> = Vec::new();
    let mut io_buffer: String = String::new();
    let mut current_depth: usize = 0;
    let mut prev_info: Option<Info> = None;
    let mut stdout = io::stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All));
    let stay_in_file = false;
    let mut mem_mode: bool = false;

    loop {
   //     stdout.execute(terminal::Clear(terminal::ClearType::All));
        stdout.execute(crossterm::cursor::MoveTo(0, 0));
        let mut skip_interaction = false;
        let mut skip_exec: bool = false;

        let origin_info = &tokens[pc].origin_info;
        let info = if origin_info.len() == 0 {
            &tokens[pc].info
        } else {
            let file_name = &origin_info[0].1.file; // Suboptimal

            let mut deepest_in_file_depth = 999999;
            if stay_in_file {
                for (i, x) in origin_info.iter().enumerate() {
                    if x.1.file == *file_name {
                        deepest_in_file_depth = i;
                    }
                }
            }

            current_depth = current_depth.min(origin_info.len() - 1).min(deepest_in_file_depth);
            &origin_info[current_depth].1
        };

        /*
           if let Some(prev) = &prev_info {
               if cur == prev {
                   skip_interaction = true;
               }
           }
        */
        prev_info = Some(info.clone());

        if !skip_interaction {
            println!("{current_instruction_idx}, {}", instruction_history.len());

           // asm_instruction!(info, "depth Instruction");
            if !mem_mode {
                display(info, pc, &mem);

            } else {
                mem_view::draw_mem(mem, pc);
            }
            //mem_view::draw_mem(mem, pc);
            println!("{}", io_buffer);

            loop {
                match read().unwrap() {
                    Event::Key(event) => {
                        if event.kind == KeyEventKind::Press {
                            // Only works on windows??
                            match event.code {
                                KeyCode::Char(c) => match c {
                                    ' ' => {
                                        stdout.execute(terminal::Clear(terminal::ClearType::All));
                                        mem_mode = !mem_mode;
                                        skip_exec = true;
                                    
                                    },
                                    _ => {}
                                },
                                KeyCode::Right => {
                                    if current_instruction_idx + 1 < instruction_history.len() {
                                        pc = apply_historic_instruction(
                                            mem,
                                            &instruction_history[current_instruction_idx],
                                            &mut io_buffer,
                                        );
                                    }
                                    current_instruction_idx += 1;
                                }

                                KeyCode::Left => {
                                    if current_instruction_idx >= 1 {
                                        current_instruction_idx -= 1;
                                        pc = revert_historic_instruction(
                                            mem,
                                            &instruction_history[current_instruction_idx],
                                            &mut io_buffer,
                                        );
                                    }
                                }
                                KeyCode::Down => {
                                    current_depth += 1;
                                    skip_exec = true;
                                }
                                KeyCode::Up => {
                                    if current_depth > 0 {
                                        current_depth -= 1;
                                    }
                                    skip_exec = true;
                                }
                                _ => {}
                            }
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        if current_instruction_idx >= instruction_history.len() && !skip_exec {
            let result = interpreter::interpret_single(mem, pc);
            let result = match result {
                Ok(e) => e,
                Err(e) =>  {
                    current_instruction_idx -= 1;
                    pc = revert_historic_instruction(
                        mem,
                        &instruction_history[current_instruction_idx - 1],
                        &mut io_buffer,
                    );
                    error(e, pc, tokens);
                    let mut inp: String = String::new();
                    io::stdin().read_line(&mut inp);
                    continue;
                   // panic!();
                }
            };
            let (new_pc, io_operation, instruction_history_item) = result;
            pc = new_pc;
            match io_operation {
                IOOperation::Char(c) => {
                    io_buffer.push(c);
                    //   print!("{c}");
                    //      io::stdout().flush();
                }
                IOOperation::Debug(i) => {
                    let i = i as i16;
                    println!("{i}");
                    io::stdout().flush();
                }
                IOOperation::Halt => {
                    return;
                }
                IOOperation::BreakPoint => {
                    in_debugging_mode = true;
                }
                IOOperation::Perf => {
                    todo!()
                }
                IOOperation::None => {}
            }
            instruction_history.push(instruction_history_item);
            current_instruction_idx = instruction_history.len();
        }
    }
}
