use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

const PLAINTEXT_EXTENSION: &str = "sblx";
const BINARY_EXTENSION: &str = "bin";

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum FileType {
    Debuggable,
    Plaintext,
    Binary,
}

enum InputFileType {
    Sublang,
    Debuggable,
    Plaintext,
    Binary,
}

impl FileType {
    fn extension(&self) -> String {
        match self {
            FileType::Debuggable => "dsblx",
            FileType::Plaintext => PLAINTEXT_EXTENSION,
            FileType::Binary => BINARY_EXTENSION,
        }
        .to_owned()
    }
    fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().trim() {
            "dsblx" => Some(FileType::Debuggable),
            PLAINTEXT_EXTENSION => Some(FileType::Plaintext),
            BINARY_EXTENSION => Some(FileType::Binary),
            _ => None,
        }
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

#[derive(PartialEq, Debug)]
pub struct OutputFile {
    file_type: FileType,
    file_name: String,
}
impl OutputFile {
    /*
     * Only filetype: SBLX, BIN....
     * Only filename: File
     * File + Filetype: File.sblx
     */
    pub fn new(argument: &Option<String>, module_name: String) -> Option<Self> {
        let argument = argument.clone()?;

        let file_type = FileType::from_str(&argument);

        // Filetype
        if let Some(file_type) = file_type {
            return Some(OutputFile {
                file_type,
                file_name: module_name,
            });
        };

        let path = Path::new(&argument).to_path_buf();

        // File + Filetype
        if let Some(x) = path.extension() {
            let file_type = FileType::from_str(x.to_str().unwrap());
            if let Some(file_type) = file_type {
                return Some(OutputFile {
                    file_name: path.with_extension("").to_string_lossy().to_string(),
                    file_type,
                });
            }
        }

        // File only
        Some(OutputFile {
            file_type: FileType::Binary,
            file_name: argument,
        })
    }
}

pub fn to_file(dat: &[u16], output: Option<OutputFile>) {
    let output = match output {
        Some(out) => out,
        None => return,
    };
    let binding = to_text(dat).as_bytes().to_vec();
    let data = match output.file_type {
        FileType::Debuggable => todo!(),
        FileType::Plaintext => binding,

        FileType::Binary => to_bin(dat),
    };

    let mut path = Path::new(&output.file_name).to_path_buf();
    path.set_extension(output.file_type.extension());
    let mut file = File::create(path).unwrap();
    file.write_all(&data).unwrap();
}

/// Binary format is in Big Endian
pub fn to_bin(data: &[u16]) -> Vec<u8> {
    let mut u8data: Vec<u8> = Vec::with_capacity(data.len() * 2);

    for i in data {
        u8data.push((i >> 8) as u8);
        u8data.push((i & 0xFF) as u8);
    }

    u8data
}

/// Binary format is in Big Endian
pub fn from_bin(data: &[u8]) -> Vec<u16> {
    let mut u16data: Vec<u16> = Vec::with_capacity((data.len() / 2) + 1); // Is +1 necessary

    for i in (0..data.len()).step_by(2) {
        u16data.push(((data[i] as u16) << 8) + (data[i + 1] as u16));
    }

    u16data
}

pub fn get_target_and_module_name(argument: Option<String>) -> (PathBuf, String) {
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

    (target, module)
}

pub fn get_mode(target: PathBuf) {
    let ext = target.extension().unwrap();
}

mod tests {
    use super::*;

    #[test]
    fn output_file() {
        let result = OutputFile::new(&None, "Sublib".to_owned());
        assert_eq!(result, None);

        let result = OutputFile::new(&Some("sblx".to_owned()), "mod".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_name: "mod".to_owned(),
                file_type: FileType::Plaintext
            })
        );
        let result = OutputFile::new(&Some("bin".to_owned()), "mod2".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_name: "mod2".to_owned(),
                file_type: FileType::Binary
            })
        );

        let result = OutputFile::new(&Some("test".to_owned()), "mod3".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_name: "test".to_owned(),
                file_type: FileType::Binary
            })
        );

        let result = OutputFile::new(&Some("abc/file.dsblx".to_owned()), "mod4".to_owned());
        assert_eq!(
            result,
            Some(OutputFile {
                file_name: "abc/file".to_owned(),
                file_type: FileType::Debuggable
            })
        );
        let result = OutputFile::new(
            &Some("abc/defg/file.dsblx.hello".to_owned()),
            "mod4".to_owned(),
        );
        assert_eq!(
            result,
            Some(OutputFile {
                file_name: "abc/defg/file.dsblx.hello".to_owned(),
                file_type: FileType::Binary
            })
        );
    }

    #[test]
    fn target_and_module_name() {
        let (target, module) = get_target_and_module_name(None);
        assert_eq!(target, Path::new("./Main.sbl"));
        assert_eq!(module, "asa");

        let (target, module) =
            get_target_and_module_name(Some("subleq/tests/Fibonacci.sbl".to_owned()));
        assert_eq!(target, Path::new("subleq/tests/Fibonacci.sbl"));
        assert_eq!(module, "Fibonacci");

        let (target, module) = get_target_and_module_name(Some("subleq".to_owned()));
        assert_eq!(target, Path::new("subleq/Main.sbl"));
        assert_eq!(module, "subleq");
    }
}
