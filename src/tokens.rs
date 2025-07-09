
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabelOffset {
    Char(char),
    Int(i32)
}
#[derive(Debug, PartialEq, Eq, Clone)]

pub struct Info {
    pub start_char: i32,
    pub length: i32,
    pub line_number: i32,
    pub file: String,
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

    BracedLabelDefinition {info: Info, name: String, data: IntOrString},

    Mult {info: Info},
    NamespaceEnd {info: Info},

}


impl Token {
    pub fn size(&self) -> i32 {
        match self {
            Token::DecLiteral { .. } | Token::Relative {..} | Token::Label {.. } | Token::BracedLabelDefinition { .. } => 1,
            Token::HexLiteral { .. } | Token::CharLiteral {..} | Token::StrLiteral {..} => todo!(),
            _ => 0,
        }
    }
    pub fn get_info(&self) -> &Info {
        match self {
            Token::DecLiteral { info, .. } => info,
            Token::HexLiteral { info, .. } => info,
            Token::LabelArrow { info, .. } => info,
            Token::Subleq { info } => info,
            Token::Label { info, .. } => info,
            Token::LabelDefinition { info, .. } => info,
            Token::Relative { info, .. } => info,
            Token::Scope { info } => info,
            Token::Unscope { info } => info,
            Token::CharLiteral { info, .. } => info,
            Token::StrLiteral { info, .. } => info,
            Token::MacroDeclaration { info, .. } => info,
            Token::MacroBodyStart { info } => info,
            Token::MacroBodyEnd { info } => info,
            Token::MacroCall { info, .. } => info,
            Token::Namespace { info, .. } => info,
            Token::BraceOpen { info } => info,
            Token::BraceClose { info } => info,
            Token::Linebreak { info } => info,
            Token::BracedLabelDefinition { info, .. } => info,
            Token::Mult { info } => info,
            Token::NamespaceEnd { info} => info,
        }
    }
}
