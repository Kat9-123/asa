use clap::Parser;
use once_cell::sync::OnceCell;

static ARGS: OnceCell<Args> = OnceCell::new();
#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum FeedbackLevel {
    Debug,
    #[default]
    Notes,
    Warn,
    Error,
}
impl ToString for FeedbackLevel {
    // Required method
    fn to_string(&self) -> String {
        match &self {
            FeedbackLevel::Debug => "debug".to_owned(),
            FeedbackLevel::Notes => "notes".to_owned(),
            FeedbackLevel::Warn => "warn".to_owned(),
            FeedbackLevel::Error => "error".to_owned(),
        }
    }
}
/// Advanced Subleq Assembler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// File to assemble
    pub target: String,
    #[arg(short,long, default_value_t = FeedbackLevel::Notes)]
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
    /// Out file.
    #[arg(short, long, default_value = "")]
    out_file: String,
    /// Disable type checking
    #[arg(long, default_value_t = false)]
    pub disable_type_checking: bool,
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
