use crate::tokens::Token;
use crate::tokens::TokenVariant;
use crate::{codegen, lexer, parser, print_debug, println_debug};

pub fn assemble(text: &str, path: String) -> (Vec<u16>, Vec<Token>) {
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
