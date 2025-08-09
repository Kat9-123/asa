use std::fs;
use std::path::{Path, PathBuf};

use log::error;

use crate::feedback::terminate;

/// Does some basic and safe sanitisation. It's fine to apply it multiple times.
pub fn generic_sanitisation(text: &str) -> String {
    text.replace("\r\n", "\n").replace("\t", "    ")
}

pub fn include_imports(text: &str, currently_imported: &mut Vec<PathBuf>) -> String {
    let cleaned_string: String = generic_sanitisation(text);

    let str_split = cleaned_string.split("\n").collect::<Vec<&str>>();
    let mut split: Vec<String> = str_split.into_iter().map(|x| x.to_string()).collect();

    let mut i = 0;
    while i < split.len() {
        if split[i].is_empty() {
            i += 1;
            continue;
        }

        if !split[i].starts_with('#') {
            i += 1;
            continue;
        }
        let path_str = &format!("./subleq/{}", &split[i][1..]); // Format is temp

        let mut path = Path::new(path_str).to_path_buf();

        // When trying to import a folder, it looks for a .sbl file with the same name inside of the folder
        if path.is_dir() {
            path.push(path.clone().file_stem().unwrap());
        }

        if path.extension().is_none() {
            path.set_extension("sbl");
        }
        // Write out the full path into the import character
        split[i] = "#".to_owned() + path.clone().to_str().unwrap();

        if currently_imported.contains(&path) {
            split.insert(i + 1, "/".to_owned()); // '/' is used to delimit EOF

            i += 2;
            continue;
        }
        currently_imported.push(path.clone());

        let contents = fs::read_to_string(&path).unwrap_or_else(|_| {
            error!("Couldn't include the file: '{:?}'", path);
            terminate();
            unreachable!()
        });
        let contents = generic_sanitisation(&contents);
        let contents = include_imports(&contents, currently_imported);

        split.insert(i + 1, contents);
        split.insert(i + 2, "/".to_owned()); // '/' is used to delimit EOF
        i += 3;
    }
    let mut result = String::new();
    for i in split {
        result.push_str(&i);
        result.push('\n');
    }

    result
}
