
#[cfg(test)]
mod tests {
    use std::fs;

    use log::{debug, LevelFilter, info};
    use simple_logger::SimpleLogger;

    use crate::{assemble, codegen};

    use super::*;

    #[test]
    fn test_full() {
        SimpleLogger::new().init().unwrap();
        log::set_max_level(LevelFilter::Debug);
        let paths = fs::read_dir("./subleq/tests").unwrap();

        for path in paths {
            let p = path.unwrap().path();
            let p = p.to_str().unwrap();

            if !&p.ends_with(".sbl") {
                continue;
            }

            info!("Name: {}", p);

            let contents = fs::read_to_string(&p).unwrap();
            let result = codegen::to_text(assemble(contents));

            let mut sblx_path = p[..p.len() - 4].to_string();
            sblx_path.push_str(".sblx");
            println!("{sblx_path}");
            let expected = fs::read_to_string(sblx_path).unwrap();
            assert_eq!(result, expected);


         //   let should_be = fs::read_to_string(.unwrap();            

        }
    } 
}