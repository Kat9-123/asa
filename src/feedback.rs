use colored::Colorize;
enum Type {
    INFO,
    WARN,
    ERROR,
}



#[macro_export]
macro_rules! asm_error {
    ($info:expr, $($arg:tt)*) => {
        {
            crate::feedback::asm_err(format!($($arg)*), $info, file!(), line!());
            panic!();
        }
    };
}


#[macro_export]
macro_rules! asm_warn {
    ($info:expr, $($arg:tt)*) => {
        {
            crate::feedback::asm_warning(format!($($arg)*), $info, file!(), line!());
        }
    };
}

#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {
        format!("\n{} {}", colored::Colorize::blue("Hint:"), format!($($arg)*))
    };
}


#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}


use std::{collections::btree_map::Range, env, fs, process::exit};

pub(crate) use asm_error;
pub(crate) use asm_warn;

use crate::tokens::Info;


fn get_file_contents() -> String {
    let args: Vec<String> = env::args().collect();

    let file_path = format!("./subleq/{}", args[1]);

   // info!("{file_path}");
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    return contents;
}

pub fn asm_err(msg: String, info: &Info, file_name: &str, line: u32) {
    log::error!("({file_name}:{line}) Line {}", info.line_number);

    asm_msg(msg, info, Type::ERROR);
    panic!();
}


fn asm_msg(msg: String, info: &Info, t: Type) {
    let contents =  get_file_contents();
    let lines = contents.lines().collect::<Vec<&str>>();
    println!("{: >4} | {}", format!("{}", info.line_number - 2).bright_cyan(), lines[(info.line_number - 3) as usize]);
    println!("{: >4} | {}", format!("{}", info.line_number - 1).bright_cyan(),  lines[(info.line_number - 2) as usize]);

    match t {
        Type::ERROR => println!("{: >4} | {}",  format!("{}", info.line_number).red() , lines[(info.line_number - 1) as usize]),
        Type::WARN =>  println!("{: >4} | {}",  format!("{}", info.line_number).yellow() , lines[(info.line_number - 1) as usize]),
        _ => todo!()
    }
    // Very very messy
    let start = info.start_char;
    let end = info.end_char;

    
    print!("      ");
    for _ in 0..start-1 {
        print!(" ");
    }
    for _ in start-1..end-1 {
        match t {
            Type::ERROR => print!("{}", "~".red()),
            Type::WARN =>print!("{}", "~".yellow()),
            _ => todo!()
        }
    }
    println!();

    println!("{}", msg.bold());
    println!();
}

pub fn asm_warning(msg: String, info: &Info, file_name: &str, line: u32) {
    log::warn!("({file_name}:{line}) Line {}", info.line_number);
    asm_msg(msg, info, Type::WARN);
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