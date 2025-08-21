use std::time::Instant;

use crate::println_silenceable;
use crate::tokens::Token;
use crate::tokens::TokenVariant;
use crate::{codegen, lexer, parser, print_debug, println_debug};

pub fn assemble(text: &str, path: String) -> (Vec<u16>, Vec<Token>) {
    println_silenceable!("Assembling {}", path);
    let timer = Instant::now();
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

    let (mem, tokens) = codegen::generate(tokens);
    println_silenceable!("\nAssembled in: {:.3?}", timer.elapsed());
    println_silenceable!(
        "Size: {}/{}, {:.4}%",
        mem.len(),
        0xFFFF,
        (mem.len() as f32 / 0xFFFF as f32) * 100f32
    );
    (mem, tokens)
}
