use colored::Colorize;
use std::io::{self, Write};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor
};

pub fn setup() {
    let mut stdout = io::stdout();
}

pub fn draw_mem(mem: &Vec<u16>, pc: usize) {

    /*
    println!("LEGEND");
    println!("{}", "Instruction".cyan());
    println!("{}", "a".yellow());
    println!("{}", "b".purple());
    println!("{}", "c".red());
 */ 
    let mut stdout = io::stdout();
  //  stdout.execute(cursor::MoveTo(0,0));




    print!("----  ");
    for i in 0..16 {
        print!("{}", format!("{:04X}  ", i ).bright_black())
    }

    for (i, item) in mem.iter().enumerate() {
        let s = match i {
            x if x >= pc && x <= pc + 2 => format!(" i{:0>4X}", item).cyan(),
            x if x == mem[pc] as usize => format!(" a{:0>4X}", item).yellow(),
            x if x == mem[pc + 1] as usize => format!(" b{:0>4X}", item).purple(),
            x if x == mem[pc + 2] as usize => format!(" c{:0>4X}", item).red(),
            _ => format!("  {:0>4X}", item).normal(),
        };
        if i % 16 == 0 {
            println!();
            print!("{}", format!("{:04X}", i ).bright_black())
        }


        print!("{}", s);
    }
    println!();
    println!();


}
