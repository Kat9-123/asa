use std::num::TryFromIntError;

use crate::{
    asm_error, asm_warn,
    tokens::{Token, TokenVariant},
};

pub fn generate(statements: Vec<Token>) -> (Vec<u16>, Vec<Token>) {
    let mut mem: Vec<u16> = Vec::with_capacity(statements.len());
    let mut final_tokens: Vec<Token> = Vec::with_capacity(statements.len());
    for statement in statements {
        match &statement.variant {
            TokenVariant::DecLiteral { value } => {
                let as_u16 = *value as u16;
                if (*value >> 16) > 0 {
                    asm_warn!(
                        &statement.info,
                        "Number is too large, it will equal {}",
                        as_u16
                    );
                }
                mem.push(as_u16);
                final_tokens.push(statement.clone());
            }

            // Ideally none of these would be here
            TokenVariant::Scope
            | TokenVariant::Unscope
            | TokenVariant::LabelDefinition { .. }
            | TokenVariant::Namespace { .. } // These two shouldnt be allowed
            | TokenVariant::NamespaceEnd => {
                continue;
            }
            _ => {
                asm_error!(&statement.info, "Unprocessed token",);
            }
        }
    }
    (mem, final_tokens)
}
