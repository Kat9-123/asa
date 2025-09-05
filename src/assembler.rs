//! Dispatches the lexer, parser and code generator
use std::time::Instant;

use log::LevelFilter;

use crate::println_silenceable;
use crate::tokens::Token;
use crate::tokens::TokenVariant;
use crate::{codegen, lexer, parser};

pub fn assemble(text: &str, path: String) -> (Vec<u16>, Vec<Token>) {
    println_silenceable!("Assembling {}", path);

    let timer = Instant::now();

    let tokens = lexer::tokenise(text.to_owned(), path);

    if log::max_level() >= LevelFilter::Debug {
        log::debug!("Tokens:");
        for i in &tokens {
            if let TokenVariant::Linebreak = i.variant {
                println!();
                continue;
            }
            print!("{i:?}, ");
        }
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
