use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabelOffset {
    Char(char),
    Int(i32),
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

#[derive(PartialEq, Eq, Clone)]
pub struct Token {
    pub info: Info,
    pub variant: TokenVariant,
   // pub macro_trace: Option<Vec<Info>>,
    pub origin_info: Vec<(i32, Info)>// Option<Info>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenVariant {
    DecLiteral { value: i32 },
    HexLiteral { value: String },
    LabelArrow { offset: LabelOffset },
    Subleq,
    Label { name: String },
    LabelDefinition { name: String, offset: i32 },
    Relative { offset: i32 },
    Scope,
    Unscope,
    CharLiteral { value: char },
    StrLiteral { value: String },

    MacroDeclaration { name: String },
    MacroBodyStart,
    MacroBodyEnd,

    MacroCall { name: String },
    Namespace { name: String },

    BraceOpen,
    BraceClose,

    Linebreak,

    BracedLabelDefinition { name: String, data: IntOrString },

    Asterisk,
    NamespaceEnd,
}

impl Token {
    pub fn size(&self) -> i32 {
        match self.variant {
            TokenVariant::DecLiteral { .. }
            | TokenVariant::Relative { .. }
            | TokenVariant::Label { .. }
            | TokenVariant::BracedLabelDefinition { .. } => 1,
            TokenVariant::HexLiteral { .. }
            | TokenVariant::CharLiteral { .. }
            | TokenVariant::StrLiteral { .. } => todo!(),
            _ => 0,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self.variant)
    }
}
