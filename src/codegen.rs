//! Generate a vec of executable words from a vector of tokens

use crate::{
    asm_error, asm_warn, terminate,
    tokens::{Token, TokenVariant},
};

/// Returns both a list of executable words AND their corrosponding tokens,
/// to be able to give runtime errors
pub fn generate(tokens: Vec<Token>) -> (Vec<u16>, Vec<Token>) {
    let mut mem: Vec<u16> = Vec::with_capacity(tokens.len());
    let mut final_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    for token in tokens {
        if final_tokens.len() > 0xFFF0 {
            log::error!("Program is too big");
            terminate!();
        }
        match &token.variant {
            TokenVariant::DecLiteral { value } => {
                let as_u16 = *value as u16;
                if (*value >> 16) > 0 {
                    asm_warn!(
                        &token.info,
                        "Number {} is too large, it will equal {}",
                        value,
                        as_u16
                    );
                }
                mem.push(as_u16);
                final_tokens.push(token.clone());
            }

            // Ideally none of these would be here
            TokenVariant::Scope | TokenVariant::Unscope | TokenVariant::LabelDefinition { .. } => {
                continue;
            }
            _ => {
                asm_error!(&token.info, "Unprocessed token",);
            }
        }
    }
    (mem, final_tokens)
}
