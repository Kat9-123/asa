use crate::{
    asm_error, asm_info,
    tokens::{self, Info, LabelOffset, Token, TokenVariant},
};

/// a &2 => a 3
pub fn resolve_relatives(tokens: &Vec<Token>) -> Vec<Token> {
    let mut address: i32 = 0;
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    for token in tokens {
        match token.variant {
            TokenVariant::Relative { offset } => {
                new_tokens.push(Token::with_info(
                    TokenVariant::DecLiteral {
                        value: address + offset,
                    },
                    &token,
                ));
            }
            _ => new_tokens.push(token.clone()),
        }
        address += token.size();
    }
    new_tokens
}

/// LABEL * 3 => LABEL LABEL LABEL
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

fn token_variants_to_tokens(
    token_variants: Vec<TokenVariant>,
    info: &Info,
    origin_info: &Vec<(i32, Info)>,
) -> Vec<Token> {
    token_variants
        .iter()
        .map(|x| Token {
            variant: x.clone(),
            info: info.clone(),
            origin_info: origin_info.clone(),
        })
        .collect()
}

pub fn handle_assignments(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if i + 1 < tokens.len()
            && let TokenVariant::Equals = &tokens[i + 1].variant
        {
            let label_tok = if let TokenVariant::Label { name } = &tokens[i].variant {
                &tokens[i]
            } else {
                asm_error!(&tokens[i].info, "Can only assign to a label");
            };
            match &tokens[i + 2].variant {
                TokenVariant::DecLiteral { value } if *value == 0 => {}
                _ => todo!(),
            }
            new_tokens.append(&mut token_variants_to_tokens(
                vec![
                    label_tok.variant.clone(),
                    label_tok.variant.clone(),
                    TokenVariant::Relative { offset: 2 },
                    TokenVariant::Linebreak,
                    label_tok.variant.clone(),
                    TokenVariant::LabelArrow {
                        offset: tokens::LabelOffset::Int(0),
                    },
                    TokenVariant::DecLiteral { value: 0 },
                    TokenVariant::Linebreak,
                ],
                &tokens[i + 1].info,
                &tokens[i + 1].origin_info,
            ));
            i += 3;
            continue;
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
            new_tokens.push(Token::with_info(
                TokenVariant::LabelDefinition {
                    name: name.clone(),
                    offset: label_offset,
                },
                &tokens[idx + 1],
            ));

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
            (0, TokenVariant::Relative { offset: 4 }),
            (1, TokenVariant::Relative { offset: -4 }),
            (2, TokenVariant::Relative { offset: 123 }),
            (3, TokenVariant::Relative { offset: 0 }),
            (4, TokenVariant::Relative { offset: 1 }),
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            (0, TokenVariant::DecLiteral { value: 4 }),
            (1, TokenVariant::DecLiteral { value: -3 }),
            (2, TokenVariant::DecLiteral { value: 125 }),
            (3, TokenVariant::DecLiteral { value: 3 }),
            (4, TokenVariant::DecLiteral { value: 5 }),
        ]);
        let output = resolve_relatives(&mut input);
        assert_eq!(output, expected);
    }
    #[test]
    fn test_assignment() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                0,
                TokenVariant::Label {
                    name: "label".to_string(),
                },
            ),
            (0, TokenVariant::Equals),
            (0, TokenVariant::DecLiteral { value: 0 }),
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                0,
                TokenVariant::Label {
                    name: "label".to_string(),
                },
            ),
            (
                0,
                TokenVariant::Label {
                    name: "label".to_string(),
                },
            ),
            (0, TokenVariant::Relative { offset: 2 }),
            (0, TokenVariant::Linebreak),
            (
                0,
                TokenVariant::Label {
                    name: "label".to_string(),
                },
            ),
            (
                0,
                TokenVariant::LabelArrow {
                    offset: tokens::LabelOffset::Int(0),
                },
            ),
            (0, TokenVariant::DecLiteral { value: 0 }),
            (0, TokenVariant::Linebreak),
        ]);
        let output = handle_assignments(&input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_mult() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            (0, TokenVariant::Asterisk),
            (
                1,
                TokenVariant::Label {
                    name: "label".to_owned(),
                },
            ),
            (2, TokenVariant::DecLiteral { value: 10 }),
            (3, TokenVariant::Asterisk),
            (4, TokenVariant::DecLiteral { value: 5 }),
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            (0, TokenVariant::Asterisk),
            (
                1,
                TokenVariant::Label {
                    name: "label".to_owned(),
                },
            ),
            (2, TokenVariant::DecLiteral { value: 10 }),
            (2, TokenVariant::DecLiteral { value: 10 }),
            (2, TokenVariant::DecLiteral { value: 10 }),
            (2, TokenVariant::DecLiteral { value: 10 }),
            (2, TokenVariant::DecLiteral { value: 10 }),
        ]);
        let output = expand_mults(&mut input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_fix_instructions_and_collapse_label_definitions() {
        let mut input: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                0,
                TokenVariant::Label {
                    name: "label".to_owned(),
                },
            ),
            (
                1,
                TokenVariant::LabelArrow {
                    offset: tokens::LabelOffset::Int(0),
                },
            ),
            (
                2,
                TokenVariant::Label {
                    name: "a".to_owned(),
                },
            ),
            (3, TokenVariant::Subleq),
            (
                4,
                TokenVariant::Label {
                    name: "b".to_owned(),
                },
            ),
            (5, TokenVariant::Linebreak),
            (
                6,
                TokenVariant::Label {
                    name: "b".to_owned(),
                },
            ),
            (
                7,
                TokenVariant::Label {
                    name: "a".to_owned(),
                },
            ),
            (
                8,
                TokenVariant::Label {
                    name: "c".to_owned(),
                },
            ),
            (9, TokenVariant::Linebreak),
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                1,
                TokenVariant::LabelDefinition {
                    name: "label".to_owned(),
                    offset: 0,
                },
            ),
            (
                4,
                TokenVariant::Label {
                    name: "b".to_owned(),
                },
            ),
            (
                2,
                TokenVariant::Label {
                    name: "a".to_owned(),
                },
            ),
            (5 + 3, TokenVariant::Relative { offset: 1 }),
            (
                6,
                TokenVariant::Label {
                    name: "b".to_owned(),
                },
            ),
            (
                7,
                TokenVariant::Label {
                    name: "a".to_owned(),
                },
            ),
            (
                8,
                TokenVariant::Label {
                    name: "c".to_owned(),
                },
            ),
        ]);
        let output = fix_instructions_and_collapse_label_definitions(&mut input);
        assert_eq!(output, expected);
    }
}
