//! Aval lexer, AST, parser and the diagnostic type shared by every stage.
#![forbid(unsafe_code)]

pub mod ast;
pub mod diag;
pub mod lexer;
pub mod parser;

pub use diag::{Diag, Pos};
pub use parser::{parse, ParseOut};
