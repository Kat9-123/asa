use crate::tokens::{LabelOffset, Token};



#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Instruction { a: Token, b: Token, c: Token },
    Control { x: Token },
    LabelDefinition {label: Token, offset: i32},
    Literal { x: Token },
}

impl Statement {
    pub fn size(&self) -> i32 {
        match self {
            Statement::Instruction { .. } => 3,
            Statement::LabelDefinition { .. } => 0,
            Statement::Control { .. } => 0,
            Statement::Literal { .. } => 1,
        }
    }
}




pub fn separate_statements(tokens: &Vec<Token>) -> Vec<Statement> {
    let mut statements: Vec<Statement> = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        if let Token::Linebreak {..} = tokens[idx]  {
            idx += 1;
            continue;
        }
        match tokens[idx] {
            Token::Scope { ..} | Token::Unscope { ..} | Token::Namespace {.. } => {
                statements.push(Statement::Control {
                    x: tokens[idx].clone(),
                });
                idx += 1;
                continue;
            }

            _=> {}
        }


        if idx + 2 < tokens.len() && let Token::LabelArrow {info, offset  } = &tokens[idx + 1]   {
            let label_offset =
            match offset {
                LabelOffset::Char(x) => match x {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    _ => unreachable!(),
                }
                LabelOffset::Int(x) => *x
            };

            statements.push(Statement::LabelDefinition {
                label: tokens[idx].clone(),
                offset: label_offset,
            });

            idx += 2;
            continue;
        }

        if let Token::Subleq {info } = &tokens[idx + 1]   {
            if idx + 3 < tokens.len() && let Token::Linebreak {info} = &tokens[idx + 3] { // Maybe something else as tokens[idx + 3]
                statements.push(Statement::Instruction {
                    // Subleq has a and b flipped
                    a: tokens[idx + 2].clone(),
                    b: tokens[idx].clone(),
                    c: Token::Relative { info: info.clone(), offset: 1 },
                });
                idx += 4;
                continue;
            }
            if idx + 4 < tokens.len() {
                statements.push(Statement::Instruction {
                    // Subleq has a and b flipped
                    a: tokens[idx + 2].clone(),
                    b: tokens[idx].clone(),
                    c: tokens[idx + 3].clone(),
                });
                idx += 5;
                continue;
            }

        }

        // ???
        if let Token::Linebreak {info} = &tokens[idx] {
        } else {
            statements.push(Statement::Literal {
                x: tokens[idx].clone(),
            });
        }
        idx += 1;
    }
    return statements;
}
