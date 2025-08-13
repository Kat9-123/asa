use colored::{Color, Colorize};
use std::cell::RefCell;
use std::{fs, process::exit};

use crate::interpreter::RuntimeError;
use crate::{tokens::Info, tokens::Token};

thread_local!(static FEEDBACK_TYPE: RefCell<log::Level> = const { RefCell::new(log::Level::Debug) });

#[derive(PartialEq, Clone)]
enum Type {
    Info,
    Warn,
    Error,
    Trace,
    Details,
}
impl Type {
    pub fn colour(&self) -> Color {
        match self {
            Type::Info => Color::Blue,
            Type::Warn => Color::Yellow,
            Type::Error => Color::Red,
            Type::Trace => Color::Blue,
            Type::Details => Color::Blue,
        }
    }
}

#[cfg(not(test))]
#[macro_export]
macro_rules! asm_error {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_error(format!($($arg)*), $info, file!(), line!());
            std::process::exit(1);

        }
    };
}
#[cfg(test)]
#[macro_export]
macro_rules! asm_error {
    ($info:expr, $($arg:tt)*) => {
        {
            panic!("{}", format!($($arg)*))

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
macro_rules! asm_trace {
    ($info:expr, $($arg:tt)*) => {

        {
            $crate::feedback::_asm_trace(format!($($arg)*), $info, file!(), line!());
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

#[macro_export]
macro_rules! terminate {
    () => {
        panic!();
    };
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
macro_rules! asm_hint {
    ($($arg:tt)*) => {

        println!("     {} {} {}",":".white(), "Hint:".blue(), format!($($arg)*).white().bold())
    };
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

pub(crate) use asm_error;
pub(crate) use asm_warn;

fn get_file_contents(path: &String) -> String {
    fs::read_to_string(path).expect("Should have been able to read the file")
}

pub fn _asm_error(msg: String, info: &Info, file_name: &str, line: u32) {
    FEEDBACK_TYPE.set(log::Level::Error);

    println!();
    log::error!(
        "({file_name}:{line}) {}:{}:{}",
        info.file,
        info.line_number,
        info.start_char
    );

    asm_msg(msg, info, Type::Error);
}

/* Example:
    ERROR [asa::feedback] (src\feedback.rs:277) ./subleq/testing.sbl:52:10
      50 |
      51 | @mac a b {
      52 >     a -= b
         |          ~
         - Address at 'A' is outside of memory bounds
*/

fn asm_msg(msg: String, info: &Info, msg_type: Type) {
    let contents = get_file_contents(&info.file);
    let lines = contents.lines().collect::<Vec<&str>>();

    let file_preview_prefix = match msg_type {
        Type::Details => "     |",
        Type::Trace => "     |",
        _ => "",
    };

    // The lines above the target line
    for i in (2..4).rev() {
        if info.line_number - i >= 0 {
            println!(
                "{}{: >4} | {}",
                file_preview_prefix,
                format!("{}", info.line_number - (i - 1)).bright_cyan(),
                lines[(info.line_number - i) as usize]
            );
        }
    }

    // The target line
    let fmt = format!(
        "{}{: >4}{}{}",
        file_preview_prefix,
        format!("{}", info.line_number).color(msg_type.colour()),
        " > ".color(msg_type.colour()),
        lines[(info.line_number - 1) as usize]
    );
    if let Some(x) = &info.append_to_sourceline {
        println!("{} {}", fmt, x.purple());
    } else {
        println!("{fmt}");
    }

    // Squiggly line
    let start = info.start_char;
    let mut length = info.length;
    if length == 0 {
        length = 1;
    }
    let prefix = match msg_type {
        Type::Details => "     |     | ",
        Type::Trace => "     |     | ",
        _ => "     | ",
    };
    print!("{prefix}");

    for _ in 0..start - 1 {
        print!(" ");
    }
    for _ in 0..length {
        print!("{}", "~".color(msg_type.colour()));
    }
    println!();

    if msg_type == Type::Trace {
        return;
    }

    // Message
    let prefix = match msg_type {
        Type::Details => "     |     - ",
        _ => "     - ",
    };

    match msg_type {
        Type::Error => println!("{}{}", prefix, msg.red()),
        Type::Warn => println!("{}{}", prefix, msg.yellow()),
        Type::Info => println!("{}{}", prefix, msg.bold()),
        Type::Trace => println!("{}{}", prefix, msg.bold()),
        Type::Details => println!("{}{}", prefix, msg.bold()),
    }
}

pub fn _asm_warning(msg: String, info: &Info, file_name: &str, line: u32) {
    FEEDBACK_TYPE.set(log::Level::Warn);
    if log::max_level() < log::Level::Warn {
        return;
    }
    println!();

    log::warn!(
        "({file_name}:{line}) {}:{}:{}",
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::Warn);
}

pub fn _asm_info(msg: String, info: &Info, file_name: &str, line: u32) {
    FEEDBACK_TYPE.set(log::Level::Info);

    if log::max_level() < log::Level::Info {
        return;
    }
    println!();

    println!(
        "{} + ({file_name}:{line}) {}:{}:{}",
        "NOTE".blue(),
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::Info);
}

pub fn asm_runtime_error(e: RuntimeError, tokens: &[Token]) {
    match e {
        RuntimeError::InstructionOutOfRange(pc) => {
            asm_error_no_terminate!(&tokens[pc + 2].info, "Jump outside of memory bounds");
            for i in tokens[pc + 2].origin_info.iter().rev().skip(1) {
                asm_trace!(i, "");
            }
        }
        RuntimeError::AOutOfRange(pc) => {
            let info = &tokens[pc]
                .origin_info
                .last()
                .unwrap_or_else(|| &tokens[pc].info);
            asm_error_no_terminate!(info, "Address at 'A' is outside of memory bounds");
            for i in tokens[pc].origin_info.iter().rev().skip(1) {
                asm_trace!(i, "");
            }
        }
        RuntimeError::BOutOfRange(pc) => {
            asm_error_no_terminate!(
                &tokens[pc + 1].info,
                "Address at 'B' is outside of memory bounds"
            );
            for i in tokens[pc + 1].origin_info.iter().rev().skip(1) {
                asm_trace!(i, "");
            }
        }
    }
}

pub fn _asm_details(msg: String, info: &Info, file_name: &str, line: u32) {
    if FEEDBACK_TYPE.with(|cell| {
        let t = cell.borrow();

        log::max_level() < *t
    }) {
        return;
    }
    println!("     |");
    println!(
        "     + {} ({file_name}:{line}) {}:{}:{}",
        "Details".blue(),
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::Details);
}

pub fn _asm_trace(msg: String, info: &Info, file_name: &str, line: u32) {
    if FEEDBACK_TYPE.with(|cell| {
        let t = cell.borrow();

        log::max_level() < *t
    }) {
        return;
    }
    println!("     |");
    println!(
        "     + {} ({file_name}:{line}) {}:{}:{}",
        "Trace".blue(),
        info.file,
        info.line_number,
        info.start_char
    );
    asm_msg(msg, info, Type::Trace);
}
