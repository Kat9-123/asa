//! Generates and prints friendly feedback messages for the user
use crate::lexer;
use crate::runtimes::RuntimeError;
use crate::{tokens::Info, tokens::Token};
use colored::{Color, Colorize};
use core::fmt;
use std::cell::RefCell;
use std::fs;

thread_local!(
    /// The type/level of the current feedback message, used by sub messages
    /// to check if they have a sufficient level
    static FEEDBACK_TYPE: RefCell<log::Level> = const { RefCell::new(log::Level::Debug) });

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
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
    pub fn to_log_level(&self) -> log::Level {
        match self {
            Type::Info => log::Level::Info,
            Type::Warn => log::Level::Warn,
            Type::Error => log::Level::Error,
            Type::Trace => log::Level::Info,
            Type::Details => log::Level::Info,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = format!(
            "{: >5}",
            match self {
                Type::Info => "Note",
                Type::Warn => "WARN",
                Type::Error => "ERROR",
                Type::Trace => "Trace",
                Type::Details => "Info",
            }
        )
        .stylise(*self);
        write!(f, "{text}")
    }
}

trait Stylise {
    fn stylise(&self, msg_type: Type) -> String;
}
impl Stylise for str {
    fn stylise(&self, msg_type: Type) -> String {
        match msg_type {
            Type::Info => self.blue(),
            Type::Warn => self.yellow(),
            Type::Error => self.red(),
            Type::Trace => self.blue(),
            Type::Details => self.blue(),
        }
        .to_string()
    }
}

#[cfg(not(test))]
#[macro_export]
macro_rules! asm_error {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_msg($crate::feedback::Type::Error, format!($($arg)*), $info, file!(), line!());
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

/// Error message for an error that doesn't originate from a token
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        {
            #[cfg(debug_assertions)]
            log::error!("{} ({}:{})", format!($($arg)*), file!(), line!());
            #[cfg(not(debug_assertions))]
            log::error!("{}", format!($($arg)*), );

            std::process::exit(1);
        }
    };
}

#[macro_export]
macro_rules! asm_info {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_msg($crate::feedback::Type::Info, format!($($arg)*), $info, file!(), line!());
        }
    };
}

/// A sub message for extra details
#[macro_export]
macro_rules! asm_details {
    ($info:expr, $($arg:tt)*) => {

        {
            $crate::feedback::_asm_msg($crate::feedback::Type::Details, format!($($arg)*), $info, file!(), line!());
        }
    };
}

/// A type of sub message used for macro traces
#[macro_export]
macro_rules! asm_trace {
    ($origin_info:expr) => {{
        for i in $origin_info.iter().rev() {
            $crate::feedback::_asm_msg(
                $crate::feedback::Type::Trace,
                "".to_string(),
                i,
                file!(),
                line!(),
            );
        }
    }};
}
/// Terminate is used for FAILURES
#[macro_export]
macro_rules! terminate {
    () => {
        std::process::exit(1);
    };
}

/// Since the asm_error macro always terminates, sometimes we don't want that, if we
/// want to write more information and then terminate for example
#[macro_export]
macro_rules! asm_error_no_terminate {
    ($info:expr, $($arg:tt)*) => {
        {
            $crate::feedback::_asm_msg($crate::feedback::Type::Error, format!($($arg)*), $info, file!(), line!());
        }
    };
}

#[macro_export]
macro_rules! asm_warn {
    ($info:expr, $($arg:tt)*) => {
        {
            if ($crate::args::exist() && !$crate::args::get().warnings_are_errors) || !$crate::args::exist() {
                $crate::feedback::_asm_msg($crate::feedback::Type::Warn, format!($($arg)*), $info, file!(), line!());
            } else {
                // If we treat warnings as errors
                $crate::feedback::_asm_msg($crate::feedback::Type::Error, format!($($arg)*), $info, file!(), line!());
                std::process::exit(1);
            }
        }
    };
}

/// Display a small hint message under an asm_msg
#[macro_export]
macro_rules! asm_hint {
    ($($arg:tt)*) => {

        if $crate::feedback::sub_message_level_check() {
            println!("      {} {} {}",":".white(), "Hint:".blue(), format!($($arg)*).white().bold())

        }
    };
}
/// These prints will be silenced by the silence command line argument
#[macro_export]
macro_rules! println_silenceable {
    ($($arg:tt)*) => {
        if $crate::args::exist() && !$crate::args::get().silent {
            println!($($arg)*);
        }
    };
}
/// Read the contents of the file at the index in the FILES vector, which contains
/// all assembled files
fn get_file_contents(index: usize) -> String {
    let path = lexer::FILES.with_borrow(|v| v[index].clone());
    fs::read_to_string(&path).unwrap_or_else(|_| {
        crate::error!("Error reading file for file preview: {}", path.display());
    })
}

/// Sub messages, like Traces and Hints should not be printed if their parent message (like Warn) is not
/// printed due to the log level being too low
pub fn sub_message_level_check() -> bool {
    FEEDBACK_TYPE.with(|cell| {
        let t = cell.borrow();

        log::max_level() >= *t
    })
}

/// Prints a pretty error message for errors that happen during the assembly process.
/// This function is only for errors which are caused by a Token. For other types of error,
/// see error!(). It should generally only be used by the asm_ macros defined in this file
///
/// Example:
/// ERROR + ./subleq/testing.sbl:52:10
///    50 |
///    51 | @mac a b {
///    52 >     a -= b
///       |          ~
///       - Address at 'A' is outside of memory bounds
pub fn _asm_msg(
    msg_type: Type,
    msg: String,
    info: &Info,
    #[allow(unused)] asa_call_origin: &str,
    #[allow(unused)] asa_line_number: u32,
) {
    match msg_type {
        Type::Trace | Type::Details => {
            if !sub_message_level_check() {
                return;
            }

            println!("      |");
        }
        _ => {
            FEEDBACK_TYPE.set(msg_type.to_log_level());
            if log::max_level() < msg_type.to_log_level() {
                return;
            }
            println!();
        }
    }

    let name = lexer::FILES.with_borrow(|f| f[info.file].clone());

    let contents = get_file_contents(info.file);
    let lines = contents.lines().collect::<Vec<&str>>();

    let title_prefix = match msg_type {
        Type::Details | Type::Trace => "      + ",
        _ => "",
    };

    // Title
    print!("{title_prefix}{msg_type} + ");
    #[cfg(debug_assertions)] // We dont want to show the origin of the error inside of the assembler in release builds
    print!("({asa_call_origin}:{asa_line_number}) ");
    println!(
        "{}:{}:{}",
        name.display(),
        info.line_number,
        info.start_char
    );

    let file_preview_prefix = match msg_type {
        Type::Details | Type::Trace => "      | ",
        _ => "",
    };

    // The preview lines above the target line
    for i in (2..4).rev() {
        if info.line_number - i >= 0 {
            println!(
                "{}{: >5} | {}",
                file_preview_prefix,
                format!("{}", info.line_number - (i - 1)).bright_cyan(),
                lines[(info.line_number - i) as usize]
            );
        }
    }

    // The preview target line
    let fmt = format!(
        "{}{: >5}{}{}",
        file_preview_prefix,
        format!("{}", info.line_number).color(msg_type.colour()),
        " > ".stylise(msg_type),
        lines[(info.line_number - 1) as usize]
    );
    if let Some(x) = &info.sourceline_suffix {
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
        Type::Details | Type::Trace => "      |       | ",
        _ => "      | ",
    };
    print!("{prefix}");

    print!("{}", " ".repeat((start - 1) as usize));
    print!("{}", "~".stylise(msg_type).repeat(length as usize));
    println!();

    if msg_type == Type::Trace {
        return;
    }

    // Message
    let prefix = match msg_type {
        Type::Details => "      |       - ",
        _ => "      - ",
    };

    match msg_type {
        Type::Error => println!("{}{}", prefix, msg.red()),
        Type::Warn => println!("{}{}", prefix, msg.yellow()),
        Type::Info => println!("{}{}", prefix, msg.bold()),
        Type::Trace => println!("{}{}", prefix, msg.bold()),
        Type::Details => println!("{}{}", prefix, msg.bold()),
    }
}

/// Show a pretty trace for runtime errors
pub fn asm_runtime_error(e: RuntimeError, tokens: &Option<Vec<Token>>) {
    let (index, message) = match e {
        RuntimeError::AOutOfRange(pc) => (pc, "Address at 'A' is outside of memory bounds"),
        RuntimeError::BOutOfRange(pc) => (
            if tokens.is_some() { pc + 1 } else { pc },
            "Address at 'B' is outside of memory bounds",
        ),
        RuntimeError::COutOfRange(pc) => (
            if tokens.is_some() { pc + 2 } else { pc },
            "Jump outside of memory bounds",
        ),
        RuntimeError::Breakpoint(pc) => (if tokens.is_some() { pc + 2 } else { pc }, "Breakpoint"),
    };

    if let Some(tokens) = tokens {
        if index >= tokens.len() {
            return;
        }
        asm_error_no_terminate!(&tokens[index].info, "{message}");
        asm_trace!(&tokens[index].origin_info);
    } else {
        log::error!("{message}. PC: {index}");
    }
}
