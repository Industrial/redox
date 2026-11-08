use crate::ParseError;
use crate::parser::Parser;
use redox_ast::Program;
use redox_lexer::Token;

/// Parse an already-lexed token stream.
///
/// `src_len` is the source's byte length, used for the program's own span.
pub fn parse(toks: &[Token], src_len: usize) -> Result<Program, Vec<ParseError>> {
  Parser::new(toks).run(src_len)
}
