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
            crate::feedback::_asm_error(format!($($arg)*), $info, file!(), line!());
            std::process::exit(1);

        }
    };
}

#[macro_export]
macro_rules! asm_info {
    ($info:expr, $($arg:tt)*) => {
        {
            crate::feedback::_asm_info(format!($($arg)*), $info, file!(), line!());
        }
    };
}
#[macro_export]
macro_rules! asm_details {
    ($info:expr, $($arg:tt)*) => {
        {
            crate::feedback::_asm_details(format!($($arg)*), $info, file!(), line!());
        }
    };
}


pub fn terminate() {
    exit(1);
}

#[macro_export]
macro_rules! asm_error_no_terminate {
    ($info:expr, $($arg:tt)*) => {
        {
            crate::feedback::_asm_error(format!($($arg)*), $info, file!(), line!());

        }
    };
}




#[macro_export]
macro_rules! asm_warn {
    ($info:expr, $($arg:tt)*) => {
        {
            crate::feedback::_asm_warning(format!($($arg)*), $info, file!(), line!());
        }
    };
}

#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {
        format!("\n{} {}", colored::Colorize::blue("Hint:"), format!($($arg)*))
    };
}




use std::{collections::btree_map::Range, env, fs, path::Path, process::exit};

pub(crate) use asm_error;
pub(crate) use asm_warn;

use crate::tokens::Info;


fn get_file_contents(path: &String) -> String {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");
    return contents;
}

pub fn _asm_error(msg: String, info: &Info, file_name: &str, line: u32) {
    log::error!("({file_name}:{line}) {}:{}", info.file, info.line_number);

    asm_msg(msg, info, Type::ERROR, "");
}


fn asm_msg(msg: String, info: &Info, t: Type, prefix: &str) {
    let contents =  get_file_contents(&info.file);
    let lines = contents.lines().collect::<Vec<&str>>();
    if info.line_number - 3 >= 0 {
        println!("{}{: >4} | {}", prefix, format!("{}", info.line_number - 2).bright_cyan(), lines[(info.line_number - 3) as usize]);

    }
    if info.line_number - 2 >= 0 {
        println!("{}{: >4} | {}", prefix, format!("{}", info.line_number - 1).bright_cyan(),  lines[(info.line_number - 2) as usize]);
    }

    match t {
        Type::ERROR => println!("{}{: >4} > {}", prefix, format!("{}", info.line_number).red() , lines[(info.line_number - 1) as usize]),
        Type::WARN =>  println!("{}{: >4} > {}", prefix, format!("{}", info.line_number).yellow() , lines[(info.line_number - 1) as usize]),
        Type::INFO =>  println!("{}{: >4} > {}", prefix,  format!("{}", info.line_number).blue() , lines[(info.line_number - 1) as usize]),
    }
    let start = info.start_char;
    let length = info.length;

    print!("       ");
    for _ in 0..start-1 {
        print!(" ");
    }
    for _ in 0..length {
        match t {
            Type::ERROR => print!("{}", "~".red()),
            Type::WARN =>print!("{}", "~".yellow()),
            Type::INFO => print!("{}", "~".blue()),
        }
    }
    println!();

    println!("{}", msg.bold());
    println!();
}

pub fn _asm_warning(msg: String, info: &Info, file_name: &str, line: u32) {
    log::warn!("({file_name}:{line}) {}:{}", info.file, info.line_number);
    asm_msg(msg, info, Type::WARN, "");
}

pub fn _asm_info(msg: String, info: &Info, file_name: &str, line: u32) {
    log::info!("({file_name}:{line}) {}:{}", info.file, info.line_number);
    asm_msg(msg, info, Type::INFO, "");
}


pub fn _asm_details(msg: String, info: &Info, file_name: &str, line: u32) {
    println!("    {} ({file_name}:{line}) {}:{}", "DETAILS".blue(), info.file, info.line_number);
    asm_msg(msg, info, Type::INFO, "    ");
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