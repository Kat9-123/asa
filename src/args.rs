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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// File to assemble or execute.
    ///
    /// If not given it will search for ./Main.sbl in the
    /// current working directory.
    ///
    ///  If it's a folder it will look for ./folder/Main.sbl
    ///
    /// If a file with a .SBL extension is given it will be assembled. .BIN or .SBLX files will be executed
    pub target: Option<String>,

    /// Level of assembler and runtime feedback. Debug is used for assembler debugging!
    ///
    /// It also generates a dump.sbl file during the assembly process
    #[arg(short,long, default_value_t = FeedbackLevel::Note)]
    pub feedback_level: FeedbackLevel,

    /// Disable program execution, files will still be assembled and possibly written to disk
    #[arg(short = 'e', long, default_value_t = false)]
    pub disable_execution: bool,

    /// Run program with the debugger
    #[arg(short, long, default_value_t = false)]
    pub debugger: bool,

    /// Folder that stores libraries
    #[arg(short = 'l', long, default_value = "./subleq/libs")]
    pub libs_path: String,

    /// Disables type checking for macro arguments. Not recommended
    #[arg(short = 't', long, default_value_t = false)]
    pub disable_type_checking: bool,

    /// Output file, if not given no output be will generated.
    ///
    /// If given the file type can be:
    ///
    /// * Nothing in which case a .bin file with the module name will be generated
    ///
    /// * BIN or SBLX which will output with that file extension and the module name
    ///
    /// * A file name with extension
    #[arg(short, long, num_args = 0..=1)]
    pub output: Option<Option<String>>,

    /// Suppresses all assembler output except for errors, overrides --feedback-level. Program output will still be shown.
    #[arg(short, long, default_value_t = false)]
    pub silent: bool,

    /// Shows more notes and warning. Recommended for release builds
    #[arg(short, long, default_value_t = false)]
    pub pedantic: bool,

    /// Treat warnings as errors
    #[arg(short = 'w', long, default_value_t = false)]
    pub warnings_are_errors: bool,
}

pub fn get() -> &'static Args {
    ARGS.get().unwrap_or_else(|| {
        crate::error!("No arguments have been parsed");
    })
}

pub fn exist() -> bool {
    ARGS.get().is_some()
}

pub fn parse() {
    ARGS.set(Args::parse()).expect("Could not read args");
}
