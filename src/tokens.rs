use std::fmt;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabelOffset {
    Char(char),
    Int(i32),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Info {
    pub start_char: i32,
    pub length: i32,
    pub line_number: i32,
    pub file: String,
    pub append_to_sourceline: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]

pub enum IntOrString {
    Str(String),
    Int(i32),
}
use std::cmp::PartialEq;

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.variant == other.variant && self.info.start_char == other.info.start_char
    }
}

#[derive(Clone)]
pub struct Token {
    pub info: Info,
    pub variant: TokenVariant,
    pub origin_info: Vec<Info>, // Option<Info>
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
    Equals,

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

impl ToString for Token {
    // Required method
    fn to_string(&self) -> String {
        match &self.variant {
            TokenVariant::DecLiteral { value } => value.to_string(),
            TokenVariant::HexLiteral { value } => format!("0x{}", value),
            TokenVariant::LabelArrow { offset } => "->".to_string(),
            TokenVariant::Subleq => "-=".to_string(),
            TokenVariant::Label { name } => name.clone(),
            TokenVariant::LabelDefinition { name, offset } => format!("[{} -{}>]", name, offset),
            TokenVariant::Relative { offset } => format!("&{}", offset),
            TokenVariant::Scope => "{".to_string(),
            TokenVariant::Unscope => "}".to_string(),
            TokenVariant::CharLiteral { value } => value.to_string(),
            TokenVariant::StrLiteral { value } => value.clone(),
            TokenVariant::MacroDeclaration { name } => format!("@{}", name),
            TokenVariant::MacroBodyStart => "[".to_string(),
            TokenVariant::MacroBodyEnd => "]".to_string(),
            TokenVariant::MacroCall { name } => format!("!{}", name),
            TokenVariant::Namespace { name } => format!("#{}", name),
            TokenVariant::BraceOpen => "(".to_string(),
            TokenVariant::BraceClose => ")".to_string(),
            TokenVariant::Linebreak => "\n".to_string(),
            TokenVariant::BracedLabelDefinition { name, data } => format!("({} -> ..)", name),
            TokenVariant::Asterisk => "*".to_string(),
            TokenVariant::NamespaceEnd => "\\".to_string(),
            TokenVariant::Equals => "=".to_string(),
        }
    }
}

pub fn dump_tokens(file_name: &str, tokens: &[Token]) -> std::io::Result<()> {
    let mut buf: String = String::new();
    let mut tabs: String = String::new();
    let mut prev_newline = true;
    for tok in tokens {
        if tok.variant == TokenVariant::Unscope {
            tabs.pop();
        }
        if prev_newline {
            buf.push_str(&tabs);
        }
        prev_newline = false;
        if tok.variant == TokenVariant::Linebreak {
            prev_newline = true;
        }
        buf.push_str(&tok.to_string());
        buf.push(' ');
        if tok.variant == TokenVariant::Scope {
            tabs.push('\t');
        }
    }
    let mut file = File::create(file_name)?;
    file.write_all(buf.as_bytes())?;
    Ok(())
}

pub fn tokens_from_token_variant_vec(token_variants: Vec<(i32, TokenVariant)>) -> Vec<Token> {
    token_variants
        .iter()
        .map(|x| Token {
            info: Info {
                start_char: x.0,
                length: 0,
                line_number: 0,
                file: String::new(),
                append_to_sourceline: None,
            },
            variant: x.1.clone(),
            origin_info: Default::default(),
        })
        .collect()
}
impl Token {
    pub fn size(&self) -> usize {
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
    pub fn new(token_variant: TokenVariant) -> Self {
        Token {
            info: Default::default(),
            origin_info: Vec::new(),
            variant: token_variant,
        }
    }
    /// Create a new token with the given tokenvariant, but with the info
    /// of the given token
    pub fn with_info(token_variant: TokenVariant, token: &Token) -> Self {
        Token {
            info: token.info.clone(),
            origin_info: token.origin_info.clone(),
            variant: token_variant,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}, {}", self.variant, self.info.start_char)
    }
}
