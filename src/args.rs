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

impl ToString for OutType {
    // Required method
    fn to_string(&self) -> String {
        match self {
            OutType::SBLX => ".sblx".to_owned(),
            OutType::BIN => ".bin".to_owned(),
            OutType::None => "No outfile".to_owned(),
        }
    }
}
impl ToString for FeedbackLevel {
    // Required method
    fn to_string(&self) -> String {
        match self {
            FeedbackLevel::Debug => "debug".to_owned(),
            FeedbackLevel::Note => "note".to_owned(),
            FeedbackLevel::Warn => "warn".to_owned(),
            FeedbackLevel::Error => "error".to_owned(),
        }
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
    pub target: String,
    #[arg(short,long, default_value_t = FeedbackLevel::Note)]
    pub feedback_level: FeedbackLevel,
    /// Debug mode
    #[arg(long, default_value_t = true)]
    assembler_debug_mode: bool,

    // Run with debugger
    #[arg(long, default_value_t = false)]
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

    /// Negate Subleq IO
    #[arg(long, default_value_t = false)]
    pub negate_io: bool,
}

pub fn get() -> &'static Args {
    ARGS.get().unwrap()
}

pub fn exist() -> bool {
    ARGS.get().is_some()
}

pub fn read() {
    ARGS.set(Args::parse()).expect("Could not read args");
}
