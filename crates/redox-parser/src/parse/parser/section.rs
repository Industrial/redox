use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::atom::atom;
use crate::parse::parser::binary_op::binary_op;
use crate::parse::parser::opens_a_section::opens_a_section;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::pipeline_section::pipeline_section;
use redox_ast::{Expr, Section};
use redox_lexer::TokenKind;

/// Parse a section, if the parenthesis opens one — doc 18 §4.
///
/// Called with the `(` **already consumed**, and returns `None` when what
/// follows is an ordinary grouping, having left the cursor where it found it.
///
/// Three shapes, told apart with no more than two tokens of lookahead:
///
/// ```text
/// (.name)     a field selector
/// (+ 1)       a right section: the operator comes first
/// (1 +)       a left section: the operator comes last
/// ```
///
/// `(|> name)` was a fourth — a method reference, a section of the pipeline —
/// and plan 57 retires it, because a bare `.name` is a method awaiting its
/// receiver and lowers to the same closure. The arm survives as a refusal
/// (`RDX0116`), so the spelling names its replacement rather than failing as
/// *"expected an expression"*.
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
  let mark = p.position();

  // `(.name)` — GHC's record dot.
  //
  // **A `)` immediately after the name is the discriminator**, and since plan
  // 57 it is doing real work: a bare `.name` is a method, so `(.age)` closes
  // and `(.cmp y)` does not. This used to `expect` the `)` and propagate,
  // which made `(.cmp 1)` a hard `RDX0101` where its three siblings below
  // rewind; it rewinds too now, and the parenthesis is an ordinary group
  // around the method application inside it.
  if p.check(&TokenKind::Dot)
    && p
      .peek_at(1)
      .is_some_and(|t| matches!(t.kind, TokenKind::Ident(_)))
  {
    p.advance();
    let name = p.ident("expected a field name after `.`")?;
    if p.check(&TokenKind::RParen) {
      p.advance();
      return Ok(Some(Expr::Section {
        section: Section::Field(name),
        span: (start, p.prev_span().1),
      }));
    }
    p.rewind(mark);
    return Ok(None);
  }

  // `(|> name)` — retired by plan 57, which builds its replacement. Kept as
  // an arm rather than deleted, so the spelling names `.name` instead of
  // reaching `prefix` and failing as *"expected an expression"*.
  if p.check(&TokenKind::Pipe)
    && let Some(token) = p.peek_at(1)
    && let TokenKind::Ident(name) = &token.kind
  {
    let end = token.span.1;
    return Err(pipeline_section(name, (start, end)));
  }

  // A right section: the operator leads. The four operators that are also
  // *prefix* operators are excluded, so `(- 1)`, `(&x)`, `(*p)` and
  // `(!flag)` keep their prefix reading — see `opens_a_section`.
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
