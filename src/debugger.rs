use crate::{interpreter::{self, IOOperation, InstructionHistoryItem}, mem_view, tokens::{Info, Token}};
use std::{io::{self, *}, num::Wrapping};
use crate::asm_instruction;
use crossterm::{event::{read, Event, KeyCode, KeyEventKind}, terminal::{self, enable_raw_mode}, ExecutableCommand};
use std::{thread, time};

pub fn revert_historic_instruction(mem: &mut Vec<u16>, inst: &InstructionHistoryItem, io_buffer: &mut String) -> usize {
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

pub fn apply_historic_instruction(mem: &mut Vec<u16>, inst: &InstructionHistoryItem, io_buffer: &mut String) -> usize {
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

pub fn debug(mem: &mut Vec<u16>,
    tokens: &Vec<Token>, mut in_debugging_mode: bool
) {
    enable_raw_mode().unwrap();
    let mut pc = 0;
    let mut current_instruction_idx = 0;
    let mut instruction_history: Vec<InstructionHistoryItem> = Vec::new();
    let mut io_buffer: String = String::new();
    let mut current_depth: usize = 0;
    let mut prev_info: Option<Info> = None;

    let stay_in_file = false;

    loop {
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All));
        stdout.execute(crossterm::cursor::MoveTo(0,0));
        let mut skip_interaction = false;
        let mut skip_exec: bool = false;
    
        let info = &tokens[pc].origin_info;
        let file_name= &info[0].1.file; // Suboptimal

        let mut deepest_in_file_depth = 999999;
        if stay_in_file {
            for (i, x) in info.iter().enumerate() {
                if x.1.file == *file_name {
                    deepest_in_file_depth = i;
                }
            }
        }


        current_depth = current_depth.min(info.len() - 1).min(deepest_in_file_depth);
        let cur = &info[current_depth].1;
        /*
        if let Some(prev) = &prev_info {
            if cur == prev {
                skip_interaction = true;
            }
        }
     */
        prev_info = Some(cur.clone());



        if !skip_interaction {

            println!("{current_instruction_idx}, {}", instruction_history.len());



            asm_instruction!(&info[current_depth].1, "depth Instruction");

            println!("PC {pc}");
            //mem_view::draw_mem(mem, pc);
            println!("{}", io_buffer);

            loop {
                match read().unwrap() {
                    Event::Key(event,) => {
                        if event.kind == KeyEventKind::Press { // Only works on windows??
                            match event.code {
                                KeyCode::Char(c) => println!("You pressed: '{}'", c),
                                KeyCode::Right => {

                                    if current_instruction_idx + 1 < instruction_history.len() {
                                        pc = apply_historic_instruction(mem, &instruction_history[current_instruction_idx], &mut io_buffer);

                                    }
                                    current_instruction_idx += 1;

                                },

                                KeyCode::Left => {
                                    if current_instruction_idx >= 1 {
                                        current_instruction_idx -= 1;
                                        pc = revert_historic_instruction(mem, &instruction_history[current_instruction_idx], &mut io_buffer);
                                    }

                                },
                                KeyCode::Down => {
                                    current_depth += 1;
                                    skip_exec = true;
                                
                                },
                                KeyCode::Up => {
                                    if current_depth > 0 {
                                        current_depth -= 1;

                                    }
                                    skip_exec = true;
                                },
                                _ => {}
                            }
                            break;
                        }

                    }
                    _ => {},
                }
            }
        }
        if current_instruction_idx >= instruction_history.len() && !skip_exec {

            let result = interpreter::interpret_single(mem, pc);
            let result = match result {
                Ok(e) => e,
                Err(e) => todo!(),
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
                IOOperation::Perf => {todo!()},
                IOOperation::None => {},
            }
            instruction_history.push(instruction_history_item);
            current_instruction_idx = instruction_history.len();

        }


    }


}