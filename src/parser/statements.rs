use crate::{feedback::asm_err, tokens::{LabelOffset, Token}};






pub fn separate_statements(tokens: &Vec<Token>) -> Vec<Token> {
    let mut new_tokens: Vec<Token> = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        if let Token::Linebreak {..} = tokens[idx]  {
            idx += 1;
            continue;
        }



        if idx + 2 < tokens.len() && let Token::LabelArrow {info, offset  } = &tokens[idx + 1]   {
            let label_offset = match offset {
                LabelOffset::Char(x) => match x {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    _ => unreachable!(),
                }
                LabelOffset::Int(x) => *x
            };

            let name = match &tokens[idx] {
                Token::Label { info, name } => name,
                _ => { asm_err(format!("Only a label may precede a label arrow"), info); unreachable!() },
            };

            new_tokens.push(Token::LabelDefinition {
                info: info.clone(),
                name: name.clone(),
                offset: label_offset,
            });

            idx += 2;
            continue;
        }
        /*
            B -= A
            B -= A C

            X->B -= A
            B -= Y->A 
            X->B -= Y->A
         */

        if let Token::Subleq {info } = &tokens[idx + 1]   {
            if idx + 3 < tokens.len() && let Token::Linebreak {info} = &tokens[idx + 3] { // Maybe something else as tokens[idx + 3]
                    // Subleq has a and b flipped
                new_tokens.push(tokens[idx + 2].clone());
                new_tokens.push(tokens[idx].clone());
                new_tokens.push(Token::Relative { info: info.clone(), offset: 1 });

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
    return new_tokens;
}
