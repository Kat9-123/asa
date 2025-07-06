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
    MacroStart { name: String },
    MacroEnd,
    MacroCall { name: String },
    Namespace {name: String},

    LabelDefintion {name: String},
    
}
