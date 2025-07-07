use crate::tokens::Token;



#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Instruction { a: Token, b: Token, c: Token },
    Control { x: Token },
    LabelDefinition {label: Token},
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
        if tokens[idx] == Token::StatementEnd {
            idx += 1;
            continue;
        }
        match tokens[idx] {
            Token::Scope | Token::Unscope | Token::Namespace { .. } => {
                statements.push(Statement::Control {
                    x: tokens[idx].clone(),
                });
                idx += 1;
                continue;
            }

            _=> {}
        }


        if idx + 2 < tokens.len() && tokens[idx + 1] == Token::LabelArrow {

            statements.push(Statement::LabelDefinition {
                label: tokens[idx].clone(),
            });

            idx += 2;
            continue;
        }

        if tokens[idx + 1] == Token::Subleq {
            if idx + 3 < tokens.len() && tokens[idx + 3] == Token::StatementEnd {
                statements.push(Statement::Instruction {
                    // Subleq has a and b flipped
                    a: tokens[idx + 2].clone(),
                    b: tokens[idx].clone(),
                    c: Token::Relative { offset: 1 },
                });
                idx += 4;
                continue;
            }
            if idx + 4 < tokens.len() && tokens[idx + 4] == Token::StatementEnd {
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


        if tokens[idx] != Token::StatementEnd {
            statements.push(Statement::Literal {
                x: tokens[idx].clone(),
            });
        }
        idx += 1;
    }
    return statements;
}
