





#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use log::{LevelFilter, debug, info};
    use simple_logger::SimpleLogger;

    use crate::{
        assemble, codegen,
        interpreter::{self, interpret},
        preprocessor::generic_sanitisation,
    };

    use super::*;
    fn test_at_path(path: &str) {

        let paths = fs::read_dir(path).unwrap();

        for path in paths {
            let p = path.unwrap().path();
            let p = p.to_str().unwrap();

            if !&p.ends_with(".sbl") {
                continue;
            }

            info!("Name: {}", p);

            let contents = fs::read_to_string(&p).unwrap();
            let (mut mem, tokens) = assemble(contents, p.to_string());
            let result = codegen::to_text(&mem);

            let mut sblx_path = p[..p.len() - 4].to_string();
            sblx_path.push_str(".sblx");

            let mut out_path = p[..p.len() - 4].to_string();
            out_path.push_str(".out");

            let fp = Path::new(&sblx_path);

            if fp.is_file() {
                let expected = fs::read_to_string(sblx_path).unwrap();
                assert_eq!(result, expected);
            }

            let fp = Path::new(&out_path);
            if !fp.is_file() {
                continue;
            }
            let expected_out = fs::read_to_string(fp).unwrap();
            let expected_out = generic_sanitisation(&expected_out);
            let out = interpret(&mut mem, &tokens, true, false).unwrap();
            assert_eq!(out, expected_out);

            println!();

            //   let should_be = fs::read_to_string(.unwrap();
        }
    }
    #[test]
    fn setup() {
        SimpleLogger::new().init().unwrap();
        log::set_max_level(LevelFilter::Info);
    }
    #[test]
    fn basic() {
        test_at_path("./subleq/tests");
    }
    #[test]
    fn sublib() {
        test_at_path("./subleq/sublib/tests");
    }
}
