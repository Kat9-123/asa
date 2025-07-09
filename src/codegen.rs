use crate::{tokens::Token};

/*
fn to_bytes(tokens: Vec<u16>) {
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
pub fn to_text(data: Vec<u16>) -> String {

    let mut text: String = String::new();
    for i in data {
        text.push_str(&i.to_string());
        text.push(' ');
    }
    text.pop();

    return text;
}
pub fn generate(statements: Vec<Token>) -> (Vec<u16>, Vec<Token>) {
    let mut mem: Vec<u16> = Vec::new();
    let mut final_tokens: Vec<Token> = Vec::new();
    for statement in statements {
        match &statement {

            Token::DecLiteral {info, value} => {
                mem.push(*value as u16);
                final_tokens.push(statement.clone());
            }
            _ => {
                continue;
            }
        }
    }
    println!("{mem:?}");
    return (mem, final_tokens);

}