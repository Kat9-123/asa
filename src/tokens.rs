
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabelOffset {
    Char(char),
    Int(i32)
}
#[derive(Debug, PartialEq, Eq, Clone)]

pub struct Info {
    pub line_number: i32,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub enum IntOrString {
    Str(String),
    Int(i32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    DecLiteral { info: Info, value: i32 },
    HexLiteral { info: Info, value: String },
    LabelArrow { info: Info, offset: LabelOffset},
    Subleq {info: Info},
    Label {info: Info, name: String },
    LabelDefinition {info: Info, name: String, offset: i32},
    Relative { info: Info, offset: i32 },
    Scope {info: Info},
    Unscope {info: Info},
    CharLiteral {info: Info,  value: char },
    StrLiteral {info: Info, value: String },

    MacroDeclaration {info: Info, name: String },
    MacroBodyStart {info: Info},
    MacroBodyEnd {info: Info},

    MacroCall {info: Info, name: String },
    Namespace {info: Info, name: String},

    BraceOpen {info: Info},
    BraceClose {info: Info},

    Linebreak {info: Info},

    BracedLabelDefinition {info: Info, name: String, data: IntOrString}

}


impl Token {
    pub fn size(&self) -> i32 {
        match self {
            Token::DecLiteral { .. } | Token::Relative {..} | Token::Label {.. } | Token::BracedLabelDefinition { .. } => 1,
            Token::HexLiteral { .. } | Token::CharLiteral {..} | Token::StrLiteral {..} => todo!(),
            _ => 0,
        }
    }
}
