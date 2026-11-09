pub(crate) mod error;
pub(crate) mod failure;
pub(crate) mod max_nesting;
pub(crate) mod nesting_depth;
pub(crate) mod parser;
pub(crate) mod precedence;
pub(crate) mod source;

use crate::ParseError;
use crate::parse::max_nesting::MAX_NESTING;
use crate::parse::nesting_depth::nesting_depth;
use crate::parse::parser::Parser;
use redox_ast::Program;
use redox_diag::Code;
use redox_lexer::Token;

/// Parse an already-lexed token stream.
///
/// `src_len` is the source's byte length, used for the program's own span.
///
/// The nesting check is here rather than in `parse_source` because this is the
/// chokepoint: every caller arrives through it, so nothing can reach the
/// recursive descent unchecked.
pub fn parse(toks: &[Token], src_len: usize) -> Result<Program, Vec<ParseError>> {
  let deepest = nesting_depth(toks);
  if deepest > MAX_NESTING {
    // Reported at the start of the file rather than at the offending bracket.
    // Finding *which* bracket crossed the line would mean another scan for a
    // span nobody can act on differently — the answer is always "this file
    // nests too deeply", and it is almost always generated input rather than
    // something a person wrote.
    return Err(vec![ParseError {
      code: Code::NestingTooDeep,
      message: format!(
        "this input nests brackets {deepest} deep, and the limit is {MAX_NESTING}: \
         deeper than that overflows the parser's stack"
      ),
      span: (0, 0),
    }]);
  }
  Parser::new(toks).run(src_len)
}
