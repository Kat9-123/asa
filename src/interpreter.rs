use std::{io::{self, Write}, num::Wrapping};

use crate::{asm_error, mem_view, tokens::Token};



pub fn interpret(mem: &mut Vec<u16>, tokens: &Vec<Token>) {
    let mut programme_counter = 0;
    let mut prev_programme_counter = 0;

    loop {
      //  mem_view::draw_mem(&mem);

        let a=
        if programme_counter < mem.len() {
            mem[programme_counter] as usize
        } else {
            asm_error!(tokens[prev_programme_counter].get_info(), "Runtime errorA");
        };
        let b =
        if programme_counter + 1 < mem.len() {
            mem[programme_counter + 1] as usize
        } else {
            asm_error!(tokens[prev_programme_counter].get_info(), "Runtime errorB");
        };
        let c =
        if programme_counter + 2 < mem.len() {
            mem[programme_counter + 2] as usize
        } else {
            asm_error!(tokens[prev_programme_counter].get_info(), "Runtime errorC");
        };


        let mut result: u16 =  0;

        if b == 0xFFFF {
            result = mem[a];
            print!("{}", result as u8 as char );
            io::stdout().flush();

        } else if a == 0xFFFF {

        } else {
            if a >= mem.len() {
                asm_error!(tokens[programme_counter].get_info(), "Address A out of range");
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
        let mut buf = String::new();
    }
}
