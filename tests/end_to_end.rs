use std::fs;

use log::{LevelFilter, info};
use simple_logger::SimpleLogger;

use asa::{feedback::asm_runtime_error, *};

fn test_at_path(path: &str) {
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        let input_file = path.unwrap().path();
        let extension = input_file.extension();
        if extension.is_none() || extension.unwrap() != "sbl" {
            continue;
        }

        info!("Name: {}", input_file.display());
        println!("{}", "-".repeat(80));

        let (target, input_file_type, _module) =
            files::get_target_and_module_name(Some(input_file.to_string_lossy().to_string()));
        let (mut mem, tokens) = files::process_input_file(&target, input_file_type);
        let result = files::to_text(&mem);

        let sblx_path = input_file.with_extension("sblx");
        let out_path = input_file.with_extension("out");

        let sblx_is_file = sblx_path.is_file();
        if sblx_is_file {
            let expected = fs::read_to_string(sblx_path).unwrap();
            assert_eq!(result, expected);
        }

        if !out_path.is_file() {
            if !sblx_is_file {
                crate::error!("No .sblx or .out found for '{}'", input_file.display());
            }
            continue;
        }
        let expected_out = fs::read_to_string(out_path).unwrap();
        let expected_out = lexer::generic_sanitisation(&expected_out);
        let (result, ..) = runtimes::interpreter::interpret(&mut mem);
        let out = result.unwrap_or_else(|e| {
            asm_runtime_error(e, &tokens);
            terminate!()
        });
        assert_eq!(out, expected_out);

        println!();
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
    test_at_path("./subleq/libs/sublib/tests");
}

#[test]
fn examples() {
    test_at_path("./subleq/examples");
}
