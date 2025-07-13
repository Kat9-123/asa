use colored::Colorize;
use std::io::{self};

pub fn setup() {
    let stdout = io::stdout();
}

pub fn draw_mem(mem: &Vec<u16>, pc: usize) {

    /*
    println!("LEGEND");
    println!("{}", "Instruction".cyan());
    println!("{}", "a".yellow());
    println!("{}", "b".purple());
    println!("{}", "c".red());
 */ 
    let stdout = io::stdout();
  //  stdout.execute(cursor::MoveTo(0,0));




    print!("----  ");
    for i in 0..16 {
        print!("{}", format!("{i:04X}  " ).bright_black())
    }

    for (i, item) in mem.iter().enumerate() {
        let s = match i {
            x if x >= pc && x <= pc + 2 => format!(" i{item:0>4X}").cyan(),
            x if x == mem[pc] as usize => format!(" a{item:0>4X}").yellow(),
            x if x == mem[pc + 1] as usize => format!(" b{item:0>4X}").purple(),
            x if x == mem[pc + 2] as usize => format!(" c{item:0>4X}").red(),
            _ => format!("  {item:0>4X}").normal(),
        };
        if i % 16 == 0 {
            println!();
            print!("{}", format!("{i:04X}" ).bright_black())
        }


        print!("{s}");
    }
    println!();
    println!();


}
