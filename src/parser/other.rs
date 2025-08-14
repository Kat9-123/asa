use crate::{
    asm_error, asm_info,
    tokens::{Info, LabelOffset, Token, TokenVariant},
};

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
                i += 3; // Remember that this is the index of the ORIGINAL tokens, so plus three
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
    origin_info: &[Info],
) -> Vec<Token> {
    token_variants
        .iter()
        .map(|x| Token {
            variant: x.clone(),
            info: info.clone(),
            origin_info: origin_info.to_owned(),
        })
        .collect()
}

pub fn insert_asm_macro(macro_name: String, origin_tok: &Token, args: Vec<&Token>) -> Vec<Token> {
    let mut toks: Vec<Token> = Vec::new();
    toks.push(Token::with_info(
        TokenVariant::MacroCall { name: macro_name },
        origin_tok,
    ));

    for arg in args {
        toks.push(arg.clone());
    }

    toks
}

/// TODO
pub fn handle_assignments(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if i + 2 < tokens.len()
            && let TokenVariant::Equals = &tokens[i + 1].variant
        {
            let label_tok = if let TokenVariant::Label { .. } = &tokens[i].variant {
                &tokens[i]
            } else {
                asm_error!(
                    &tokens[i].info,
                    "The left hand side of an assignment may only be a label"
                );
            };

            let target_tok = &tokens[i + 2];
            match &target_tok.variant {
                TokenVariant::DecLiteral { value } if *value == 0 => {
                    let mut toks = insert_asm_macro(
                        "ASM::AssignZero".to_string(),
                        &tokens[i + 1],
                        vec![label_tok],
                    );
                    new_tokens.append(&mut toks);
                }
                TokenVariant::DecLiteral { .. } => {
                    let mut toks = insert_asm_macro(
                        "ASM::AssignLit".to_string(),
                        &tokens[i + 1],
                        vec![label_tok, target_tok],
                    );
                    new_tokens.append(&mut toks);
                }
                TokenVariant::Label { .. } => {
                    let mut toks = insert_asm_macro(
                        "ASM::AssignLabel".to_string(),
                        &tokens[i + 1],
                        vec![label_tok, target_tok],
                    );
                    new_tokens.append(&mut toks);
                }
                _ => asm_error!(
                    &tokens[i + 2].info,
                    "The right hand side of an assignment may only be a label or a literal"
                ),
            }
            i += 3;
            continue;
        }
        new_tokens.push(tokens[i].clone());
        i += 1;
    }

    new_tokens
}

/// This routine does two different things
/// It fixes instructions to be in the subleq format i.e: b -= a => a b &1
/// and it converts label definitions into a single token instead of the two
/// *label* and ->. At this point linebreaks are also removed, since they don't carry
/// any meaning anymore.
pub fn fix_instructions_and_collapse_label_definitions(tokens: &[Token]) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

    let mut i = 0;

    while i < tokens.len() {
        if let TokenVariant::Linebreak = tokens[i].variant {
            i += 1;
            continue;
        }

        if i + 1 < tokens.len()
            && let TokenVariant::LabelArrow { offset } = &tokens[i + 1].variant
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

            let name = match &tokens[i].variant {
                TokenVariant::Label { name } => name,
                _ => asm_error!(&tokens[i].info, "Only a label may precede a label arrow"),
            };
            new_tokens.push(Token::with_info(
                TokenVariant::LabelDefinition {
                    name: name.clone(),
                    offset: label_offset,
                },
                &tokens[i + 1],
            ));

            i += 2;
            continue;
        }

        if i + 1 < tokens.len()
            && let TokenVariant::Subleq = &tokens[i + 1].variant
        {
            /*  doesn't work, shouldn't trigger for ZERO -= ZERO...
            if let TokenVariant::Label {name} = &tokens[i].variant {
                if name.len() > 1 {
                    if name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                        asm_info!(&tokens[i].info, "Attempting to write to a label notated as being constant");
                    }
                }
            }
             */
            if i + 3 < tokens.len()
                && let TokenVariant::Linebreak = &tokens[i + 3].variant
            {
                // Subleq has a and b flipped
                let mut updated_info = tokens[i + 2].info.clone();
                updated_info.start_char += updated_info.length + 2;
                updated_info.length = 2;
                updated_info.sourceline_suffix = Some("$1".to_string());

                // Flip b and a
                new_tokens.push(tokens[i + 2].clone());
                new_tokens.push(tokens[i].clone());
                new_tokens.push(Token {
                    info: updated_info,
                    variant: TokenVariant::Relative { offset: 1 },
                    origin_info: tokens[i + 3].origin_info.clone(), // Linebreak info
                });

                i += 4;
                continue;
            }
            if i + 4 < tokens.len() {
                // Subleq has a and b flipped
                new_tokens.push(tokens[i + 2].clone());
                new_tokens.push(tokens[i].clone());
                new_tokens.push(tokens[i + 3].clone());
                if let TokenVariant::Label { name } = &tokens[i + 3].variant {
                    // This is a little hack, because macros add their own name to the label, in the format: '?MACRO?label',
                    // here we only care about the 'label' part
                    let mut split_name = name.split('?');
                    if !split_name.next_back().unwrap().starts_with('.') {
                        asm_info!(
                            &tokens[i + 3].info,
                            "Labels which are jump targets should be prefixed with a '.'"
                        );
                    }
                }
                i += 5;
                continue;
            }
        }

        new_tokens.push(tokens[i].clone());

        i += 1;
    }
    new_tokens
}

#[cfg(test)]
mod tests {

    use crate::tokens::{self, tokens_from_token_variant_vec};

    use super::*;

    #[test]
    fn test_assignment() {
        let input: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                0,
                TokenVariant::Label {
                    name: "label".to_string(),
                },
            ),
            (0, TokenVariant::Equals),
            (0, TokenVariant::DecLiteral { value: 0 }),
            (0, TokenVariant::Linebreak),
            (
                0,
                TokenVariant::Label {
                    name: "label2".to_string(),
                },
            ),
            (0, TokenVariant::Equals),
            (0, TokenVariant::DecLiteral { value: 120 }),
            (
                0,
                TokenVariant::Label {
                    name: "label_a".to_string(),
                },
            ),
            (0, TokenVariant::Equals),
            (
                0,
                TokenVariant::Label {
                    name: "label_b".to_string(),
                },
            ),
        ]);
        let expected: Vec<Token> = tokens_from_token_variant_vec(vec![
            (
                0,
                TokenVariant::MacroCall {
                    name: "ASM::AssignZero".to_string(),
                },
            ),
            (
                0,
                TokenVariant::Label {
                    name: "label".to_string(),
                },
            ),
            (0, TokenVariant::Linebreak),
            (
                0,
                TokenVariant::MacroCall {
                    name: "ASM::AssignLit".to_string(),
                },
            ),
            (
                0,
                TokenVariant::Label {
                    name: "label2".to_string(),
                },
            ),
            (0, TokenVariant::DecLiteral { value: 120 }),
            (
                0,
                TokenVariant::MacroCall {
                    name: "ASM::AssignLabel".to_string(),
                },
            ),
            (
                0,
                TokenVariant::Label {
                    name: "label_a".to_string(),
                },
            ),
            (
                0,
                TokenVariant::Label {
                    name: "label_b".to_string(),
                },
            ),
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
            (6, TokenVariant::Relative { offset: 1 }),
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
