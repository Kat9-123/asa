use std::fs;
use std::path::{Path, PathBuf};
use crate::{print_debug, println_debug};


pub fn include_imports(text: String, currently_imported: &mut Vec<PathBuf>, source_level: bool) -> String{

    let cleaned_string: String = text.replace("\r\n", "\n").replace("\t", " ");

    let str_split = cleaned_string.split("\n").collect::<Vec<&str>>();
    let mut split: Vec<String> = str_split.into_iter().map(|x| x.to_string()).collect();

    let mut i = 0;
    while i < split.len() {
        println_debug!("{:?}", split);
        if split[i].len() < 1 {
            i += 1;
            continue;
        }

        if split[i].chars().nth(0).unwrap() != '#' {
            i += 1;
            continue;
        }
        let filepath = &format!("./subleq/{}", &split[i][1..]); // Format is temp

        let mut fp = Path::new(filepath).to_path_buf();

        if fp.is_dir() {
            fp.push(fp.clone().file_stem().unwrap());
        }

        if fp.extension() == None {
            fp.set_extension("sbl");
        }
        split[i] = "#".to_owned() + fp.clone().to_str().unwrap();

        if currently_imported.contains(&fp) {
            split.insert(i + 1, "/".to_owned());

            i += 2;
            continue;
        }
        println_debug!("{:?}", fp);
        currently_imported.push(fp.clone());

        let contents = fs::read_to_string(&fp).expect("Should have been able to read the file");
        let contents = contents.replace("\r\n", "\n").replace("\t", " ");
        let contents = include_imports(contents, currently_imported, false);



        split.insert(i + 1, contents);
        split.insert(i + 2, "/".to_owned());
        i += 3;

      //  if source_level {
     //       split.insert(i, String::from("#THIS"));
     //       i += 1;
    //    }
    }
    let mut result = String::new();
    for i in split {
        result.push_str(&i);
        result.push('\n');
    }

    print_debug!("{}", result);
    return result;

}