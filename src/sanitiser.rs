use crate::symbols;

#[derive(PartialEq, Eq)]
enum Context {
    Normal,
    LineComment,
    String,
    Char,
}

fn updated_context(current_context: Context, symbol: char) -> Context {
    return match current_context {
        Context::Normal => match symbol {
            symbols::LINE_COMMENT => Context::LineComment,
            symbols::CHAR => Context::Char,
            symbols::STRING => Context::String,
            _ => Context::Normal,
        },
        Context::LineComment => match symbol {
            '\n' => Context::Normal,
            _ => Context::LineComment,
        },
        Context::String => match symbol {
            symbols::STRING => Context::Normal,
            '\n' => panic!("Multiline strings are not allowed."),
            _ => Context::String,
        },
        Context::Char => match symbol {
            symbols::CHAR => Context::Normal,
            _ => Context::Char,
        },
    };
}

pub fn sanitise(text: String) -> String {
    let basic_cleaned: String = text.replace("\r\n", "\n"); // Maybe a bad idea?

    let mut current_context: Context = Context::Normal;
    let mut result: String = "".to_string();

    for (i, c) in basic_cleaned.bytes().enumerate() {
        current_context = updated_context(current_context, c as char);
        if current_context == Context::LineComment {
            continue;
        }

        result.push(c as char)
    }
    return result;
}

