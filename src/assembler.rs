use std::path::Path;
use std::path::PathBuf;

use crate::tokens::Token;
use crate::tokens::TokenVariant;
use crate::{codegen, lexer, parser, preprocessor, print_debug, println_debug};
use log::debug;

pub fn assemble(text: &str, path: String) -> (Vec<u16>, Vec<Token>) {
    let mut currently_imported: Vec<PathBuf> = vec![Path::new(&path).to_path_buf()];

    // let with_imports = preprocessor::include_imports(text, &mut currently_imported);
    //debug!("With imports: ");
    //  print_debug!("{}", with_imports);

    let tokens = lexer::tokenise(text.to_owned(), path);
    println_debug!("Tokens:");

    for i in &tokens {
        if let TokenVariant::Linebreak = i.variant {
            println_debug!();
            continue;
        }
        print_debug!("{:?}, ", i);
    }

    let tokens = parser::parse(tokens);

    codegen::generate(tokens)
}
