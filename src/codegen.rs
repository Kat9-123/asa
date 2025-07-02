use crate::{parser::Statement, tokens::Token};

/*
fn generate_as_bytes(tokens: Vec<Token>) {
    let x = 65535_u16;
    let x_bytes = x.to_be_bytes();                  // x_bytes = [0, 0, 255, 255]
    let original_x = u16::from_be_bytes(x_bytes);
}

fn generate_as_text(tokens: Vec<Token>) {
    let mut result: String = "".to_string();
    let mut index = 0;

    for token in tokens {

        match token {
            Token::DecLiteral { value: val } => {
                result.push_str(&val.to_string());
                index += 1;
                if index % 3 == 0 {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            },
            _ => continue,//; panic!("Found a non Decimical Literal during generation phase."),
        }
    }
    let mut file = File::create("foo.txt").expect("Access denied.");
    let _ = file.write_all(result.as_bytes());
}
*/


pub fn generate(statements: Vec<Statement>) {
    let mut mem: Vec<u16> = Vec::new();
    for statement in statements {
        match statement {
            Statement::Instruction { a,b, c } => {
                if let Token::DecLiteral { value } = a {
                    mem.push(value as u16);
                }
                if let Token::DecLiteral { value } = b {
                    mem.push(value as u16);
                }
                if let Token::DecLiteral { value } = c {
                    mem.push(value as u16);
                }
            }
            Statement::ScopeControl { x } => {
                continue;
            }
            Statement::PointerDefinition { label, value: val } => {
                if let Token::DecLiteral { value } = val {
                    mem.push(value as u16);
                }
            }
            Statement::Literal { x: Token } => {
                continue;
            }
        }
    }
    println!("{mem:?}");

}