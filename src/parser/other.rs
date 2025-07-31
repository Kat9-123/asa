use crate::{
    asm_error, asm_info,
    tokens::{LabelOffset, Token, TokenVariant},
};

pub fn resolve_relatives(tokens: &Vec<Token>) -> Vec<Token> {
    let mut address: i32 = 0;
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    for token in tokens {
        match token.variant {
            TokenVariant::Relative { offset } => {
                new_tokens.push(Token {
                    info: token.info.clone(),
                    variant: TokenVariant::DecLiteral {
                        value: address + offset,
                    },
                    origin_info: token.origin_info.clone(),
                    //macro_trace: token.macro_trace.clone()
                });
            }
            _ => new_tokens.push(token.clone()),
        }
        address += token.size();
    }
    new_tokens
}




pub fn expand_mults(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if i + 1 < tokens.len()
            && let TokenVariant::Asterisk = tokens[i + 1].variant
        {
            if let TokenVariant::DecLiteral { value: count } = &tokens[i + 2].variant {
                for _ in 0..*count {
                    new_tokens.push(tokens[i].clone());
                }
                i += 3;
                continue;
            }
            // its the deref operator
        }
        new_tokens.push(tokens[i].clone());
        i += 1;
    }
    new_tokens
}

pub fn fix_instructions_and_collapse_label_definitions(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    let mut idx = 0;

    while idx < tokens.len() {
        if let TokenVariant::Linebreak = tokens[idx].variant {
            idx += 1;
            continue;
        }

        if idx + 2 < tokens.len()
            && let TokenVariant::LabelArrow { offset } = &tokens[idx + 1].variant
        {
            let label_offset = match offset {
                LabelOffset::Char(x) => match x {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    _ => unreachable!(),
                },
                LabelOffset::Int(x) => *x,
            };

            let name = match &tokens[idx].variant {
                TokenVariant::Label { name } => name,
                _ => asm_error!(&tokens[idx].info, "Only a label may precede a label arrow"),
            };

            new_tokens.push(Token {
                info: tokens[idx + 1].info.clone(),
                variant: TokenVariant::LabelDefinition {
                    name: name.clone(),
                    offset: label_offset,
                },
                origin_info: tokens[idx + 1].origin_info.clone(),
                // macro_trace: tokens[idx + 1].macro_trace.clone(),
            });

            idx += 2;
            continue;
        }

        if idx + 1 < tokens.len()
            && let TokenVariant::Subleq = &tokens[idx + 1].variant
        {
            /*  doesnt work, shouldnt trigger for ZERO -= ZERO...
            if let TokenVariant::Label {name} = &tokens[idx].variant {
                if name.len() > 1 {
                    if name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                        asm_info!(&tokens[idx].info, "Attempting to write to a label notated as being constant");
                    }
                }
            }
             */
            if idx + 3 < tokens.len()
                && let TokenVariant::Linebreak = &tokens[idx + 3].variant
            {
                // Maybe something else as tokens[idx + 3]
                // Subleq has a and b flipped

                let mut updated_info = tokens[idx + 3].info.clone();
                updated_info.start_char += updated_info.length + 3; // Clones linebreak info
                updated_info.length = 2;

                new_tokens.push(tokens[idx + 2].clone());
                new_tokens.push(tokens[idx].clone());
                new_tokens.push(Token {
                    info: updated_info,
                    variant: TokenVariant::Relative { offset: 1 },
                    origin_info: tokens[idx + 3].origin_info.clone(),
                    //   macro_trace: tokens[idx + 3].macro_trace.clone(),
                });

                idx += 4;
                continue;
            }
            if idx + 4 < tokens.len() {
                // Subleq has a and b flipped
                new_tokens.push(tokens[idx + 2].clone());
                new_tokens.push(tokens[idx].clone());
                new_tokens.push(tokens[idx + 3].clone());
                if let TokenVariant::Label { name } = &tokens[idx + 3].variant {
                    // This is a little hack, because macros add their own name to the label, in the format: '?MACRO?label',
                    // here we only care about the 'label' part
                    let mut split_name = name.split('?');
                    if !split_name.next_back().unwrap().starts_with('.') {
                        asm_info!(
                            &tokens[idx + 3].info,
                            "Labels which are jump targets should be prefixed with a '.'"
                        );
                    }
                }
                idx += 5;
                continue;
            }
        }

        new_tokens.push(tokens[idx].clone());

        idx += 1;
    }
    new_tokens
}




#[cfg(test)]
mod tests {

    use crate::tokens::{self, tokens_from_token_variant_vec};

    use super::*;

    #[test]
    fn test_relatives() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::Relative { offset: 4 },
            TokenVariant::Relative { offset: -4 },
            TokenVariant::Relative { offset: 123 },
            TokenVariant::Relative { offset: 0 },
            TokenVariant::Relative { offset: 1 },
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::DecLiteral { value: 4 },
            TokenVariant::DecLiteral { value: -3 },
            TokenVariant::DecLiteral { value: 125 },
            TokenVariant::DecLiteral { value: 3 },
            TokenVariant::DecLiteral { value: 5 },
        ]);
        let output = resolve_relatives(&mut input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_mult() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::Asterisk,
            TokenVariant::Label {name: "label".to_owned()},
            TokenVariant::DecLiteral { value: 10 },
            TokenVariant::Asterisk,
            TokenVariant::DecLiteral { value: 5},
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::Asterisk,
            TokenVariant::Label {name: "label".to_owned()},
            TokenVariant::DecLiteral { value: 10 },
            TokenVariant::DecLiteral { value: 10 },
            TokenVariant::DecLiteral { value: 10 },
            TokenVariant::DecLiteral { value: 10 },
            TokenVariant::DecLiteral { value: 10 },
        ]);
        let output = expand_mults(&mut input);
        assert_eq!(output, expected);
    }
    #[test]

    fn test_fix_instructions_and_collapse_label_definitions() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::Label { name: "label".to_owned() },
            TokenVariant::LabelArrow { offset: tokens::LabelOffset::Int(0) },
            TokenVariant::Label { name: "a".to_owned() },
            TokenVariant::Subleq,
            TokenVariant::Label { name: "b".to_owned() },
            TokenVariant::Linebreak,
            TokenVariant::Label { name: "b".to_owned() },
            TokenVariant::Label { name: "a".to_owned() },
            TokenVariant::Label { name: "c".to_owned() },
            TokenVariant::Linebreak,


        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            TokenVariant::LabelDefinition { name: "label".to_owned(), offset: 0 },
            TokenVariant::Label { name: "b".to_owned() },
            TokenVariant::Label { name: "a".to_owned() },
            TokenVariant::Relative {offset: 1},

            TokenVariant::Label { name: "b".to_owned() },
            TokenVariant::Label { name: "a".to_owned() },
            TokenVariant::Label { name: "c".to_owned() },

        ]);
        let output = fix_instructions_and_collapse_label_definitions(&mut input);
        assert_eq!(output, expected);
    }
}
