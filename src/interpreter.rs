use std::io;
use std::num::Wrapping;
use crate::mem_view;

pub fn interpret(mem: &mut Vec<u16>) {
    let mut programme_counter = 0;

    loop {
        mem_view::draw_mem(&mem);

        
        let a = mem[programme_counter] as usize;
        let b = mem[programme_counter + 1] as usize;
        let c = mem[programme_counter + 2] as usize;

        let mut result: u16 =  0;

        if b == 0xFFFF {
            result = mem[a];
            println!("{}", result as u8 as char );

        } else if a == 0xFFFF {

        } else {
            result = (Wrapping(mem[b]) - (Wrapping(mem[a]))).0;
            mem[b] = result;
        }



        if result as i16 <= 0 {
            programme_counter = c;
            if c == 0xFFFF {
                break;
            }
            continue;
        }
        programme_counter += 3;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf);
    }
}
