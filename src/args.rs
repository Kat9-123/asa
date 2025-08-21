use core::fmt;

use clap::Parser;
use once_cell::sync::OnceCell;

static ARGS: OnceCell<Args> = OnceCell::new();
#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum FeedbackLevel {
    Debug,
    #[default]
    Note,
    Warn,
    Error,
}

impl FeedbackLevel {
    pub fn to_log_level(&self) -> log::LevelFilter {
        match self {
            FeedbackLevel::Debug => log::LevelFilter::Debug,
            FeedbackLevel::Note => log::LevelFilter::Info,
            FeedbackLevel::Warn => log::LevelFilter::Warn,
            FeedbackLevel::Error => log::LevelFilter::Error,
        }
    }
}

impl fmt::Display for FeedbackLevel {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FeedbackLevel::Debug => "debug".to_owned(),
            FeedbackLevel::Note => "note".to_owned(),
            FeedbackLevel::Warn => "warn".to_owned(),
            FeedbackLevel::Error => "error".to_owned(),
        };
        write!(fmt, "{s}")
    }
}

/// Advanced Subleq Assembler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// File to assemble or execute. May be .SBL, .BIN or .SBLX
    pub target: Option<String>,

    /// Level of assembler and runtime feedback
    #[arg(short,long, default_value_t = FeedbackLevel::Note)]
    pub feedback_level: FeedbackLevel,

    /// Disable program execution
    #[arg(short = 'e', long, default_value_t = false)]
    pub disable_execution: bool,

    /// Run program with debugger
    #[arg(short, long, default_value_t = false)]
    pub debugger: bool,

    /// Folder that stores libraries
    #[arg(short = 'l', long, default_value = "./subleq/libs")]
    pub libs_path: String,

    #[arg(short = 't', long, default_value_t = false)]
    pub disable_type_checking: bool,

    /// Output file, if not given no output be generated. It can be: Nothing, a .bin file with the module name, BIN or SBLX, will output with the module name
    /// and the given file extension.
    #[arg(short, long, num_args = 0..=1)]
    pub output: Option<Option<String>>,

    /// Suppresses all assembler output except for errors, overrides --feedback-level. Program output will still be shown.
    #[arg(short, long, default_value_t = false)]
    pub silent: bool,
}

pub fn get() -> &'static Args {
    ARGS.get().unwrap_or_else(|| {
        log::error!("No arguments parsed");
        crate::terminate!();
    })
}

pub fn exist() -> bool {
    ARGS.get().is_some()
}

pub fn parse() {
    ARGS.set(Args::parse()).expect("Could not read args");
}
