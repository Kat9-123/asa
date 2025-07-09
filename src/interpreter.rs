use std::{io::{self, Write}, num::Wrapping};

use crate::{asm_details, asm_error, asm_error_no_terminate, asm_warn, feedback::terminate, mem_view, tokens::Token};


fn outside_mem_bounds_err(tokens: &Vec<Token>, prev_pc: usize) {
    asm_error_no_terminate!(tokens[prev_pc + 2].get_info(), "Jump outside of memory bounds");
    asm_details!(tokens[prev_pc].get_info(), "'A' part");
    asm_details!(tokens[prev_pc + 1].get_info(), "'B' part");
    terminate();
}

pub fn interpret(mem: &mut Vec<u16>, tokens: &Vec<Token>, return_output: bool) -> Option<String> {
    let mut programme_counter = 0;
    let mut prev_programme_counter = 0;
    let mut buf = String::new();

    loop {
      //  mem_view::draw_mem(&mem);

        let a=
        if programme_counter < mem.len() {
            mem[programme_counter] as usize
        } else {
            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };
        let b =
        if programme_counter + 1 < mem.len() {
            mem[programme_counter + 1] as usize
        } else {
            outside_mem_bounds_err(tokens, prev_programme_counter);
            unreachable!();
        };
        let c =
        if programme_counter + 2 < mem.len() {
            mem[programme_counter + 2] as usize
        } else {
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

        } else if a == 0xFFFF {

        } else {
            if a >= mem.len() {

                asm_error_no_terminate!(tokens[programme_counter].get_info(), "Address A out of range");
                asm_details!(tokens[prev_programme_counter + 1].get_info(), "'B' part");
                asm_details!(tokens[prev_programme_counter + 2].get_info(), "'C' part");
                terminate();
            }
            if b >= mem.len() {
                asm_error!(tokens[programme_counter + 1].get_info(), "Address B out of range");
            }
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }


        prev_programme_counter = programme_counter;
        if result as i16 <= 0 {
          //  println!("JUMP!");
            programme_counter = c;
            if c == 0xFFFF {
                break;
            }
            continue;
        }
        programme_counter += 3;
    }
    if return_output {
        return Some(buf);
    }
    return None;

}
