use std::cmp::PartialEq;
use std::{fmt, fs::File, io::prelude::*};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabelOffset {
    Char(char),
    Int(i32),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IntOrString {
    Str(String),
    Int(i32),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Info {
    pub start_char: i32,
    pub length: i32,
    pub line_number: i32,
    pub file: usize,
    pub sourceline_suffix: Option<String>,
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

    Equals,
    BraceOpen,
    BraceClose,

    Linebreak,

    BracedLabelDefinition { name: String, data: IntOrString },

    Asterisk,
}
#[derive(Clone)]
pub struct Token {
    /// Origin of the token
    pub info: Info,
    pub variant: TokenVariant,
    /// Origin info is used to trace a token's origin through macro calls, every time it
    /// is used as an argument, an Info gets appended to this vec.
    pub origin_info: Vec<Info>,
}

impl Token {
    /// Size in subleq memory of a token.
    pub fn size(&self) -> usize {
        use TokenVariant::*;
        match self.variant {
            DecLiteral { .. } | Relative { .. } | Label { .. } | BracedLabelDefinition { .. } => 1,

            HexLiteral { .. } | CharLiteral { .. } | StrLiteral { .. } => {
                unreachable!("These variants should already have been processed")
            }

            _ => 0,
        }
    }

    /// Create a new token with the given token variant, but with the info
    /// of the given token
    pub fn with_info(token_variant: TokenVariant, token: &Token) -> Self {
        Token {
            info: token.info.clone(),
            origin_info: token.origin_info.clone(),
            variant: token_variant,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        // HACK
        self.variant == other.variant
            && self.info.start_char == other.info.start_char
            && self.info.line_number == other.info.line_number
            && self.info.file == other.info.file
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self.variant)
    }
}

/// Only really used for debugging
impl fmt::Display for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenVariant::*;
        let s = match &self.variant {
            DecLiteral { value } => value.to_string(),
            TokenVariant::HexLiteral { value } => format!("0x{value}"),
            LabelArrow { .. } => "->".to_string(),
            Subleq => "-=".to_string(),
            Label { name } => name.clone(),
            LabelDefinition { name, offset } => format!("[{name} -{offset}>]"),
            Relative { offset } => format!("&{offset}"),
            Scope => "{".to_string(),
            Unscope => "}".to_string(),
            CharLiteral { value } => value.to_string(),
            StrLiteral { value } => value.clone(),
            MacroDeclaration { name } => format!("@{name}"),
            MacroBodyStart => "[".to_string(),
            MacroBodyEnd => "]".to_string(),
            MacroCall { name } => format!("!{name}"),
            Namespace { name } => format!("#{name}"),
            BraceOpen => "(".to_string(),
            BraceClose => ")".to_string(),
            Linebreak => "\n".to_string(),
            BracedLabelDefinition { name, .. } => format!("({name} -> ..)"),
            Asterisk => "*".to_string(),
            Equals => "=".to_string(),
        };
        write!(fmt, "{s}")
    }
}

/// Used for debugging, dumps tokens to an sbl file
pub fn dump_tokens(tokens: &[Token]) -> std::io::Result<()> {
    let mut buf: String = String::new();
    let mut tabs: String = String::new();
    let mut prev_newline = true;
    for tok in tokens {
        if tok.variant == TokenVariant::Unscope {
            for _ in 0..4 {
                tabs.pop();
            }
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
            tabs.push_str("    ");
        }
    }
    let mut file = File::create("dump.sbl")?;
    file.write_all(buf.as_bytes())?;
    Ok(())
}

/// Used for testing. It makes a list of tokens from a list of TokenVariants with an ID
/// Info and Origin info are set to defaults and are irrelevant for testing, except for start_char, which is repurposed to be an ID,
/// used by test cases to check from which input the output tokens came from.
pub fn tokens_from_token_variant_vec(token_variants: Vec<(i32, TokenVariant)>) -> Vec<Token> {
    token_variants
        .iter()
        .map(|x| Token {
            info: Info {
                start_char: x.0,
                length: 0,
                line_number: 0,
                file: 0,
                sourceline_suffix: None,
            },
            variant: x.1.clone(),
            origin_info: Default::default(),
        })
        .collect()
}
