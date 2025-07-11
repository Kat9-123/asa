use std::{io::{self, Write}, num::Wrapping};

use log::info;

use crate::{asm_details, asm_error, asm_error_no_terminate, asm_info, asm_warn, feedback::terminate, mem_view, tokens::Token};


fn outside_mem_bounds_err(tokens: &Vec<Token>, prev_pc: usize) {
    asm_error_no_terminate!(&tokens[prev_pc + 2].info, "Jump outside of memory bounds");
    asm_details!(&tokens[prev_pc].info, "'A' part");
    asm_details!(&tokens[prev_pc + 1].info, "'B' part");
    terminate();
}


struct InstructionLog {
    pub pc: usize,
    pub jumped: bool
}

fn trace(instruction_logs: &Vec<InstructionLog>, tokens: &Vec<Token>) {
    for i in instruction_logs {
        info!("TRACE");
        asm_details!(&tokens[i.pc].info, "'A' part");
        asm_details!(&tokens[i.pc + 1].info, "'B' part");
        if i.jumped {
            asm_details!(&tokens[i.pc + 2].info, "'C' part. JUMPED");
        } else {
            asm_details!(&tokens[i.pc + 2].info, "'C' part. DIDN'T JUMP");

        }
        println!();
    }
}

pub fn interpret(mem: &mut Vec<u16>, tokens: &Vec<Token>, return_output: bool) -> Option<String> {
    let mut programme_counter = 0;
    let mut prev_programme_counter = 0;
    let mut buf = String::new();
    let mut instruction_logs: Vec<InstructionLog> = Vec::new();
    loop {
        //mem_view::draw_mem(&mem);

        let a=
        if programme_counter < mem.len() {
            mem[programme_counter] as usize
        } else {
            trace(&instruction_logs, tokens);
            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };
        let b =
        if programme_counter + 1 < mem.len() {
            mem[programme_counter + 1] as usize
        } else {
            trace(&instruction_logs, tokens);

            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };
        let c =
        if programme_counter + 2 < mem.len() {
            mem[programme_counter + 2] as usize
        } else {
            trace(&instruction_logs, tokens);

            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };


        let mut result: u16 =  0;

        if b == 0xFFFF {
            result = mem[a];
            let ch = result as u8 as char;
            buf.push(ch);
            print!("{}", ch );
            io::stdout().flush();

        } else if b == 0xFFFE {
            result = mem[a];
            let ch = (result as i16).to_string();
            buf.push_str(&ch);
            println!("{}", ch );
            io::stdout().flush();
        }else if a == 0xFFFF {

        } else {
            if a >= mem.len() {
                trace(&instruction_logs, tokens);

                asm_error_no_terminate!(&tokens[programme_counter].info, "Address A out of range");
                asm_details!(&tokens[prev_programme_counter + 1].info, "'B' part");
                asm_details!(&tokens[prev_programme_counter + 2].info, "'C' part");
                terminate();
            }
            if b >= mem.len() {
                asm_error!(&tokens[programme_counter + 1].info, "Address B out of range");
            }
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }


        prev_programme_counter = programme_counter;


        let mut jumped = false;
        if result as i16 <= 0 {
            jumped = true;
          //  println!("JUMP!");
            programme_counter = c;
            if c == 0xFFFF {
                break;
            }
        } else {
            programme_counter += 3;
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
    return None;

}
