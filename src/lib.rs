#![doc = include_str!("../README.md")]
//! ##
//! It is best to view **Introduction to sublang** the following on <https://github.com/Kat9-123/asa/blob/master/Sublang.md>
//! so you get at least a modicum of syntax highlighting
#![doc = include_str!("../Sublang.md")]
pub mod args;
pub mod assembler;
pub mod codegen;
pub mod feedback;
pub mod files;
pub mod lexer;
pub mod mem_view;
pub mod parser;
pub mod runtimes;
pub mod symbols;
pub mod tokens;
pub mod utils;
