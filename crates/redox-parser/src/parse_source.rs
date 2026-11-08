use crate::{ParseFailure, parse};
use redox_ast::Program;

/// Lex and parse `src`.
pub fn parse_source(src: &str) -> Result<Program, ParseFailure> {
  let toks = redox_lexer::lex(src).map_err(ParseFailure::Lex)?;
  parse(&toks, src.len()).map_err(ParseFailure::Parse)
}
