use colored::Colorize;
#[derive(PartialEq)]
enum Type {
    INFO,
    WARN,
    ERROR,
    DETAILS,
    INSTRUCTION,
    SUB_INSTRUCTION,
}
#[macro_export]
macro_rules! asm_sub_instruction {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_sub_instruction(format!($($arg)*), $info, file!(), line!());
        }
    }
}

#[macro_export]
macro_rules! asm_error {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_error(format!($($arg)*), $info, file!(), line!());
            std::process::exit(1);

        }
    };
}

#[macro_export]
macro_rules! asm_info {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_info(format!($($arg)*), $info, file!(), line!());
        }
    };
}
#[macro_export]
macro_rules! asm_details {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_details(format!($($arg)*), $info, file!(), line!());
        }
    };
}

#[macro_export]
macro_rules! asm_instruction {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_instruction(format!($($arg)*), $info, file!(), line!());
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
            $crate::feedback::_asm_error(format!($($arg)*), $info, file!(), line!());

        }
    };
}

#[macro_export]
macro_rules! asm_warn {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_warning(format!($($arg)*), $info, file!(), line!());
        }
    };
}

#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {

        format!("\n     {} {} {}",":".white(), "Hint:".blue(), format!($($arg)*).white())
    };
}

use std::{fs, process::exit};

pub(crate) use asm_error;
pub(crate) use asm_warn;

use crate::tokens::Info;

fn get_file_contents(path: &String) -> String {
    fs::read_to_string(path).expect("Should have been able to read the file")
}

pub fn _asm_error(msg: String, info: &Info, file_name: &str, line: u32) {
    println!();
    log::error!(
        "({file_name}:{line}) {}:{}:{}",
        info.file,
        info.line_number,
        info.start_char
    );

    asm_msg(msg, info, Type::ERROR, false);
}



fn asm_msg(msg: String, info: &Info, msg_type: Type, sub_msg: bool) {
    let contents = get_file_contents(&info.file);
    let lines = contents.lines().collect::<Vec<&str>>();
    let prefix = if !sub_msg { "" } else { "     |" };

    //  println!("{:?}",info);
    if msg_type != Type::SUB_INSTRUCTION && info.line_number - 3 >= 0 {
        println!(
            "{}{: >4} | {}",
            prefix,
            format!("{}", info.line_number - 2).bright_cyan(),
            lines[(info.line_number - 3) as usize]
        );
    }
    if msg_type != Type::SUB_INSTRUCTION && info.line_number - 2 >= 0 {
        println!(
            "{}{: >4} | {}",
            prefix,
            format!("{}", info.line_number - 1).bright_cyan(),
            lines[(info.line_number - 2) as usize]
        );
    }

    match msg_type {
        Type::ERROR => println!(
            "{}{: >4}{}{}",
            prefix,
            format!("{}", info.line_number).red(),
            " > ".red(),
            lines[(info.line_number - 1) as usize]
        ),
        Type::WARN => println!(
            "{}{: >4}{}{}",
            prefix,
            format!("{}", info.line_number).yellow(),
            " > ".yellow(),
            lines[(info.line_number - 1) as usize]
        ),
        Type::INFO => println!(
            "{}{: >4}{}{}",
            prefix,
            format!("{}", info.line_number).blue(),
            " > ".blue(),
            lines[(info.line_number - 1) as usize]
        ),
        Type::INSTRUCTION => println!(
            "{}{: >4}{}{}",
            prefix,
            format!("{}", info.line_number).blue(),
            " > ".blue(),
            lines[(info.line_number - 1) as usize]
        ),
        Type::DETAILS => println!(
            "{}{: >4}{}{}",
            prefix,
            format!("{}", info.line_number).blue(),
            " > ".blue(),
            lines[(info.line_number - 1) as usize]
        ),
        Type::SUB_INSTRUCTION => println!(
            "{}{: >4}{}{}",
            prefix,
            format!("{}", info.line_number).blue(),
            " > ".bright_cyan(),
            lines[(info.line_number - 1) as usize]
        ),
    }
    if msg_type == Type::INSTRUCTION && info.line_number + 1 < lines.len() as i32 {
        println!(
            "{}{: >4} | {}",
            prefix,
            format!("{}", info.line_number + 1).bright_cyan(),
            lines[(info.line_number) as usize]
        );
    }
    let start = info.start_char;
    let mut length = info.length;
    if length == 0 {
        length = 1;
    }
    let prefix = if !sub_msg { "     | " } else { "     |     | " };

    if msg_type != Type::INSTRUCTION {
        print!("{prefix}");

        for _ in 0..start - 1 {
            print!(" ");
        }
        for _ in 0..length {
            match msg_type {
                Type::ERROR => print!("{}", "~".red()),
                Type::WARN => print!("{}", "~".yellow()),
                Type::INFO => print!("{}", "~".blue()),
                Type::INSTRUCTION => {}
                Type::DETAILS => print!("{}", "~".blue()),
                Type::SUB_INSTRUCTION => print!("{}", "~".bright_cyan()),
            }
        }
        println!();
    }

    let prefix = if !sub_msg { "     - " } else { "     |     - " };

    match msg_type {
        Type::ERROR => println!("{}{}", prefix, msg.red()),
        Type::WARN => println!("{}{}", prefix, msg.bold().yellow()),
        Type::INFO => println!("{}{}", prefix, msg.bold()),
        Type::DETAILS => println!("{}{}", prefix, msg.bold()),
        Type::INSTRUCTION => println!("{}{}", prefix, msg.bold()),
        Type::SUB_INSTRUCTION => {}
    }
}

pub fn _asm_warning(msg: String, info: &Info, file_name: &str, line: u32) {
    println!();

    log::warn!(
        "({file_name}:{line}) {}:{}:{}",
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::WARN, false);
}

pub fn _asm_sub_instruction(msg: String, info: &Info, file_name: &str, line: u32) {
    println!("     |");
    println!(
        "     + {} ({file_name}:{line}) {}:{}:{}",
        msg.to_string().bright_cyan(),
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::SUB_INSTRUCTION, true);
}

pub fn _asm_info(msg: String, info: &Info, file_name: &str, line: u32) {
    println!();

    log::info!(
        "({file_name}:{line}) {}:{}:{}",
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::INFO, false);
}

pub fn _asm_details(msg: String, info: &Info, file_name: &str, line: u32) {
    println!("     |");
    println!(
        "     + {} ({file_name}:{line}) {}:{}:{}",
        "DETAILS".blue(),
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::INFO, true);
}

pub fn _asm_instruction(msg: String, info: &Info, file_name: &str, line: u32) {
    println!();

    println!(
        "{} ({file_name}:{line}) {}:{}:{}",
        "INSTRUCTION".blue(),
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::INSTRUCTION, false);
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
