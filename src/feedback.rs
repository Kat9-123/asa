use colored::Colorize;
enum Type {
    INFO,
    WARN,
    ERROR,
}

#[macro_export]
macro_rules! asm_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
        panic!();
    };
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}


use std::{collections::btree_map::Range, env, fs};

pub(crate) use asm_error;

use crate::tokens::Info;
fn send() {

}

fn get_file_contents() -> String {
    let args: Vec<String> = env::args().collect();

    let file_path = format!("./subleq/{}", args[1]);

    info!("{file_path}");
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    return contents;
}

pub fn asm_err(msg: String, info: &Info) {
    log::error!("Line {}", info.line_number);

    asm_msg(msg, info);
    panic!();
}


fn asm_msg(msg: String, info: &Info) {
    let contents =  get_file_contents();
    let lines = contents.lines().collect::<Vec<&str>>();
    println!("{: >4} | {}", format!("{}", info.line_number - 2).cyan(), lines[(info.line_number - 3) as usize]);
    println!("{: >4} | {}", format!("{}", info.line_number - 1).cyan(),  lines[(info.line_number - 2) as usize]);
    println!("{: >4} | {}",  format!("{}", info.line_number).cyan() , lines[(info.line_number - 1) as usize]);
    print!("       ");
    for _ in 0..info.start_char-1 {
        print!(" ");
    }
    for _ in info.start_char-1..info.end_char-1 {
        print!("{}", "~".red());
    }
    println!();

    println!("{}", msg.bold());
    println!();
}

pub fn asm_warn(msg: String, info: &Info) {
    log::warn!("Line {}", info.line_number);
    asm_msg(msg, info);
}

#[macro_export]
macro_rules! println_debug {
    ($($arg:tt)*) => {
        if log::max_level() == log::LevelFilter::Debug {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {
        if log::max_level() == log::LevelFilter::Debug {
            print!($($arg)*);
        }
    };
}