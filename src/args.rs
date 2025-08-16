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

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum OutType {
    SBLX,
    BIN,

    #[default]
    None,
}

impl fmt::Display for OutType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OutType::SBLX => "sblx".to_owned(),
            OutType::BIN => "bin".to_owned(),
            OutType::None => "none".to_owned(),
        };
        write!(fmt, "{s}")
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

/// Advanced Subleq Assembler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// File to assemble
    pub target: Option<String>,

    #[arg(short,long, default_value_t = FeedbackLevel::Note)]
    pub feedback_level: FeedbackLevel,

    // Run with debugger
    #[arg(short, long, default_value_t = false)]
    pub debugger: bool,
    /// Root path. This is the path includes are resolved from.
    #[arg(long, default_value = "")]
    root_path: String,

    /// Disable type checking
    #[arg(long, default_value_t = false)]
    pub disable_type_checking: bool,

    /// Out file type
    #[arg(long, default_value_t = OutType::None)]
    pub out_file_type: OutType,
}

pub fn get() -> &'static Args {
    ARGS.get().unwrap()
}

pub fn exist() -> bool {
    ARGS.get().is_some()
}

pub fn parse() {
    ARGS.set(Args::parse()).expect("Could not read args");
}
