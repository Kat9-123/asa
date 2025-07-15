use crate::{
    asm_error,
    tokens::{LabelOffset, Token, TokenVariant},
};

pub fn fix_instructions_and_collapse_label_definitions(tokens: &Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();

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
            /*  doesnt work, shouldnt trigger for b -= ZERO...
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

                idx += 5;
                continue;
            }
        }

        new_tokens.push(tokens[idx].clone());

        idx += 1;
    }
    new_tokens
}
