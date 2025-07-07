use std::fs;

use crate::sanitiser;




pub fn include_imports(text: String, currently_imported: &mut Vec<String>, source_level: bool) -> String{

    let str_split = text.split("\n").collect::<Vec<&str>>();
    let mut split: Vec<String> = str_split.into_iter().map(|x| x.to_string()).collect();

    let mut i = 0;
    while i < split.len() {
        println!("{:?}", split);
        if split[i].len() < 1 {
            i += 1;
            continue;
        }

        if split[i].chars().nth(0).unwrap() != '#' {
            i += 1;
            continue;
        }
        let filepath = &split[i][1..];
        println!("{}", &filepath[filepath.len()-4..]);

        let mut new_fp = String::from(filepath);
        if &filepath[filepath.len()-4..] != ".sbl" {
            new_fp.push_str(".sbl");
        }
        if currently_imported.contains(&new_fp) {
            split.remove(i);

            continue;
        }

        currently_imported.push(new_fp.clone());

        let contents = fs::read_to_string(new_fp).expect("Should have been able to read the file");
        let contents = sanitiser::sanitise(contents);
        let contents = include_imports(contents, currently_imported, false);

        split.insert(i + 1, contents);
        i += 2;

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

    print!("{}", result);
    return result;

}