use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::{assembler, println_silenceable, terminate, tokens::Token};

const PLAINTEXT_EXTENSION: &str = "sblx";
const BINARY_EXTENSION: &str = "bin";

impl InputFileType {
    fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().trim() {
            "sbl" => Some(InputFileType::Sublang),
            PLAINTEXT_EXTENSION => Some(InputFileType::Plaintext),
            BINARY_EXTENSION => Some(InputFileType::Binary),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputFileType {
    Sublang,
    Plaintext,
    Binary,
}

#[derive(Clone, Debug, PartialEq)]
enum OutputFileType {
    Plaintext,
    Binary,
}

impl OutputFileType {
    fn extension(&self) -> String {
        match self {
            OutputFileType::Plaintext => PLAINTEXT_EXTENSION,
            OutputFileType::Binary => BINARY_EXTENSION,
        }
        .to_owned()
    }
    fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().trim() {
            PLAINTEXT_EXTENSION => Some(OutputFileType::Plaintext),
            BINARY_EXTENSION => Some(OutputFileType::Binary),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct OutputFile {
    /// Example: .sblx, .bin, etc.
    file_type: OutputFileType,
    /// Example: ./folder/filename
    file_base: PathBuf,
}
impl OutputFile {
    /// The argument given can be of four types:
    ///
    /// * None, in which case None is returned
    /// * Only a filetype, like 'sblx' or 'bin', the file will become module_name.filetype
    /// * Only a path without extension, like ./my_folder/my_file, the file will become path_given.bin
    /// * A path with extension.
    pub fn new(argument: &Option<Option<String>>, module_name: String) -> Option<Self> {
        let argument = argument.clone()?;

        let argument = if let Some(arg) = argument {
            arg
        } else {
            // No name or Filetype was given
            return Some(OutputFile {
                file_type: OutputFileType::Binary,
                file_base: Path::new(&module_name).to_path_buf(),
            });
        };

        // The argument is only a filetype
        if let Some(file_type) = OutputFileType::from_str(&argument) {
            return Some(OutputFile {
                file_type,
                file_base: Path::new(&module_name).to_path_buf(),
            });
        };

        let path = Path::new(&argument).to_path_buf();

        // File + Filetype
        if let Some(ext) = path.extension() {
            let file_type = OutputFileType::from_str(ext.to_str().unwrap());
            if let Some(file_type) = file_type {
                return Some(OutputFile {
                    file_base: path.with_extension(""),
                    file_type,
                });
            }
        }

        // File only
        Some(OutputFile {
            file_type: OutputFileType::Binary,
            file_base: path,
        })
    }
}

pub fn to_text(data: &[u16]) -> String {
    let mut text: String = String::new();
    for i in data {
        text.push_str(&i.to_string());
        text.push(' ');
    }
    text.pop();

    text
}

pub fn from_text(text: &str) -> Vec<u16> {
    text.split_ascii_whitespace()
        .map(|val| {
            val.parse::<u16>()
                .unwrap_or_else(|_| crate::error!("Invalid u16 in input file"))
        })
        .collect()
}

/// Binary format is in Big Endian
pub fn to_bytes(data: &[u16]) -> Vec<u8> {
    let mut u8data: Vec<u8> = Vec::with_capacity(data.len() * 2);

    for i in data {
        u8data.push((i >> 8) as u8);
        u8data.push((i & 0xFF) as u8);
    }

    u8data
}

/// Binary format is in Big Endian
pub fn from_bytes(data: &[u8]) -> Vec<u16> {
    let mut u16data: Vec<u16> = Vec::with_capacity((data.len() / 2) + 1); // Is +1 necessary

    for i in (0..data.len()).step_by(2) {
        u16data.push(((data[i] as u16) << 8) + (data[i + 1] as u16));
    }

    u16data
}

pub fn to_file(data: &[u16], output: Option<OutputFile>) {
    let output = match output {
        Some(out) => out,
        None => return,
    };
    let bytes = match output.file_type {
        OutputFileType::Plaintext => to_text(data).as_bytes().to_vec(),
        OutputFileType::Binary => to_bytes(data),
    };

    let mut path = output.file_base;
    path.set_extension(output.file_type.extension());
    let mut file =
        File::create(path).unwrap_or_else(|e| crate::error!("Failed to create sblx file. {e}"));
    file.write_all(&bytes)
        .unwrap_or_else(|e| crate::error!("Failed to write to sblx file. {e}"));
}
/// Reads and processes the target file, returning its memory and if possible the tokens associated with it.
/// The assembler can take three types of input files:
/// * .sbl files will be assembled
/// * .bin and .sblx files will only be read
pub fn process_input_file(
    target: &PathBuf,
    input_file_type: InputFileType,
) -> (Vec<u16>, Option<Vec<Token>>) {
    match input_file_type {
        InputFileType::Sublang => {
            let contents = fs::read_to_string(target);
            let contents = contents.unwrap_or_else(|e| {
                log::error!("Error reading file: {target:?}. {e}");
                terminate!();
            });
            println_silenceable!("Assembling {}", target.display());
            let timer = Instant::now();
            let (mem, tokens) =
                assembler::assemble(&contents, target.to_str().unwrap().to_string());
            let tokens = Some(tokens);
            println_silenceable!("\nAssembled in: {:.3?}", timer.elapsed());
            println_silenceable!(
                "Size: {}/{}, {:.4}%",
                mem.len(),
                0xFFFF,
                (mem.len() as f32 / 0xFFFF as f32) * 100f32
            );

            (mem, tokens)
        }
        InputFileType::Binary => {
            let contents = fs::read(target);
            let contents = contents.unwrap_or_else(|e| {
                log::error!("Error reading file: {target:?}. {e}");
                terminate!();
            });
            (from_bytes(&contents), None)
        }
        InputFileType::Plaintext => {
            let contents = fs::read_to_string(target);
            let contents = contents.unwrap_or_else(|e| {
                log::error!("Error reading file: {target:?}. {e}");
                terminate!();
            });
            (from_text(&contents), None)
        }
    }
}

/// TODO Process the target argument for the assembler. It returns
pub fn get_target_and_module_name(argument: Option<String>) -> (PathBuf, InputFileType, String) {
    let target = argument.unwrap_or_else(|| ".".to_string());
    let cwd = env::current_dir().unwrap();

    let mut target_path = Path::new(&target).to_path_buf();

    let module = if target == "." {
        cwd.file_name()
    } else {
        target_path.file_stem()
    }
    .unwrap()
    .to_string_lossy()
    .to_string();
    let target = if target_path.is_dir() {
        target_path.push("Main.sbl");
        target_path
    } else {
        target_path
    };
    let input_file_type = InputFileType::from_str(
        target
            .extension()
            .unwrap_or_else(|| {
                log::error!("Target file does not have a file extension");
                terminate!();
            })
            .to_str()
            .unwrap(),
    )
    .unwrap_or_else(|| {
        log::error!("Target file has an invalid file extension");
        terminate!();
    });
    (target, input_file_type, module)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn output_file() {
        let result = OutputFile::new(&None, "Sublib".to_owned());
        assert_eq!(result, None);

        let result = OutputFile::new(&Some(Some("sblx".to_owned())), "mod".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_base: Path::new("mod").to_path_buf(),
                file_type: OutputFileType::Plaintext
            })
        );
        let result = OutputFile::new(&Some(Some("bin".to_owned())), "mod2".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_base: Path::new("mod2").to_path_buf(),
                file_type: OutputFileType::Binary
            })
        );

        let result = OutputFile::new(&Some(Some("test".to_owned())), "mod3".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_base: Path::new("test").to_path_buf(),
                file_type: OutputFileType::Binary
            })
        );

        let result = OutputFile::new(&Some(Some("abc/file.sblx".to_owned())), "mod4".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_base: Path::new("abc/file").to_path_buf(),
                file_type: OutputFileType::Plaintext
            })
        );
        let result = OutputFile::new(
            &Some(Some("abc/defg/file.sblx.hello".to_owned())),
            "mod4".to_owned(),
        );
        assert_eq!(
            result,
            Some(OutputFile {
                file_base: Path::new("abc/defg/file.sblx.hello").to_path_buf(),
                file_type: OutputFileType::Binary
            })
        );

        let result = OutputFile::new(&Some(None), "mod5".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_base: Path::new("mod5").to_path_buf(),
                file_type: OutputFileType::Binary
            })
        );
    }

    #[test]
    fn target_and_module_name() {
        let (target, input_file_type, module) = get_target_and_module_name(None);
        assert_eq!(target, Path::new("./Main.sbl"));
        assert_eq!(module, "asa");
        assert_eq!(input_file_type, InputFileType::Sublang);
        let (target, input_file_type, module) =
            get_target_and_module_name(Some("subleq/tests/Fibonacci.sbl".to_owned()));
        assert_eq!(target, Path::new("subleq/tests/Fibonacci.sbl"));
        assert_eq!(module, "Fibonacci");
        assert_eq!(input_file_type, InputFileType::Sublang);
        let (target, input_file_type, module) =
            get_target_and_module_name(Some("subleq".to_owned()));
        assert_eq!(target, Path::new("subleq/Main.sbl"));
        assert_eq!(module, "subleq");
        assert_eq!(input_file_type, InputFileType::Sublang);
    }
}
