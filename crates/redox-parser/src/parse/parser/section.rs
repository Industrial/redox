use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::atom::atom;
use crate::parse::parser::binary_op::binary_op;
use crate::parse::parser::opens_a_section::opens_a_section;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::{Expr, Section};
use redox_lexer::TokenKind;

/// Parse a section, if the parenthesis opens one — doc 18 §4.
///
/// Called with the `(` **already consumed**, and returns `None` when what
/// follows is an ordinary grouping, having left the cursor where it found it.
///
/// Four shapes, told apart with no more than two tokens of lookahead:
///
/// ```text
/// (.name)     a field selector
/// (|> name)   a method reference — a section of the pipeline
/// (+ 1)       a right section: the operator comes first
/// (1 +)       a left section: the operator comes last
/// ```
///
/// The left section is the only one needing a speculative read, and it reads
/// an **atom** rather than a full expression — which is also what doc 18 §2.1
/// says an argument is, so the restriction is the one already in the grammar
/// rather than a new one. `(1 + 2 *)` is not a section, and nothing in §4
/// suggests it should be.
///
/// `(- 1)` is **never** a section: doc 18 §4 makes it negation always, which
/// is what retires doc 03 decision 50's `(-1)`-versus-`(- 1)` adjacency rule.
/// Subtraction has no section; the lambda is `\\x -> x - 1`. The same holds
/// for `&`, `*` and `!`, which are prefix operators too — see
/// `opens_a_section`.
pub(crate) fn section(p: &mut Parser, start: usize) -> Result<Option<Expr>, ParseError> {
  // `(.name)` — GHC's record dot.
  if p.check(&TokenKind::Dot)
    && p
      .peek_at(1)
      .is_some_and(|t| matches!(t.kind, TokenKind::Ident(_)))
  {
    p.advance();
    let name = p.ident("expected a field name after `.`")?;
    p.expect(&TokenKind::RParen, "expected `)` to close the section")?;
    return Ok(Some(Expr::Section {
      section: Section::Field(name),
      span: (start, p.prev_span().1),
    }));
  }

  // `(|> name)` — a method reference, derived from the pipeline rather than
  // added as a sigil of its own.
  if p.check(&TokenKind::Pipe)
    && p
      .peek_at(1)
      .is_some_and(|t| matches!(t.kind, TokenKind::Ident(_)))
  {
    p.advance();
    let name = p.ident("expected a method name after `|>`")?;
    p.expect(&TokenKind::RParen, "expected `)` to close the section")?;
    return Ok(Some(Expr::Section {
      section: Section::Method(name),
      span: (start, p.prev_span().1),
    }));
  }

  // A right section: the operator leads. The four operators that are also
  // *prefix* operators are excluded, so `(- 1)`, `(&x)`, `(*p)` and
  // `(!flag)` keep their prefix reading — see `opens_a_section`.
  let mark = p.position();
  if opens_a_section(&p.peek().kind) {
    if let Some(op) = binary_op(p) {
      let operand = parse_expr(p, 0)?;
      p.expect(&TokenKind::RParen, "expected `)` to close the section")?;
      return Ok(Some(Expr::Section {
        section: Section::Right {
          op,
          operand: Box::new(operand),
        },
        span: (start, p.prev_span().1),
      }));
    }
    p.rewind(mark);
  }

  // A left section: the operator trails, so the operand is read first and
  // the read is abandoned if no `op )` follows.
  let Ok(operand) = atom(p) else {
    p.rewind(mark);
    return Ok(None);
  };
  if let Some(op) = binary_op(p)
    && p.check(&TokenKind::RParen)
  {
    p.advance();
    return Ok(Some(Expr::Section {
      section: Section::Left {
        op,
        operand: Box::new(operand),
      },
      span: (start, p.prev_span().1),
    }));
  }

  p.rewind(mark);
  Ok(None)
}
