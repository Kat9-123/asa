#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    DecLiteral { value: i32 },
    HexLiteral { value: String },
    LabelArrow,
    Subleq,
    StatementEnd,
    Label { name: String },
    Relative { offset: i32 },
    Scope,
    Unscope,
    CharLiteral { value: char },
    StrLiteral { value: String },

    MacroDeclaration { name: String },
    MacroBodyStart,
    MacroBodyEnd,

    MacroCall { name: String },
    Namespace {name: String},    
}
