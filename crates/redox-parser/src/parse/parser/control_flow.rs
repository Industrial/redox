use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::condition::condition;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::pattern_alt::pattern_alt;
use redox_ast::StmtKind;
use redox_diag::Code;
use redox_lexer::{Keyword, TokenKind};

/// Parse a control-flow statement, if one starts here — doc 01 §8 and doc 03
/// §7.7, plan 20.
///
/// ```text
/// if c            while c         for x in xs      loop
///   …               …               …               …
/// else            until c         break 'l v      continue 'l
///   …               …
/// ```
///
/// Returns `None` when the cursor is on something else, so `Parser::stmt` can
/// ask first and fall through to an expression statement.
///
/// A **label** precedes the loop keyword — `'outer while …` — which is Rust's
/// own order with the colon dropped, and is why the label is read here rather
/// than inside each loop's parser.
pub(crate) fn control_flow(p: &mut Parser) -> Result<Option<StmtKind>, ParseError> {
  // `'name` before a loop keyword. Doc 03 §7.2: *"labels apply equally to
  // loops, so `break 'outer` and `continue 'outer` reach an enclosing loop."*
  let label = match &p.peek().kind {
    TokenKind::Label(name) => {
      let name = name.clone();
      p.advance();
      Some(name)
    }
    _ => None,
  };

  if p.check(&TokenKind::Kw(Keyword::While)) || p.check(&TokenKind::Kw(Keyword::Until)) {
    let negated = p.check(&TokenKind::Kw(Keyword::Until));
    let keyword_span = p.peek_span();
    p.advance();
    let condition = condition(p)?;
    // `until` lowers to `while !(c)`, and a pattern test cannot be negated
    // that way: `while !(let Some(v) = x)` is not Rust, and the nearest Rust
    // — `while !matches!(x, Some(_))` — binds nothing, so it cannot be what a
    // binding construct meant. Plan 22 §1.3: report it rather than lower it
    // to something plausible.
    let reject = negated && condition.has_pattern();
    let body = p.statement_block("expected an indented loop body")?;
    // Reported **after** the body is read, so recovery resumes at the next
    // statement rather than inside a block it never consumed. Returning early
    // produced a second, cascading "expected an expression" against the loop's
    // own first line — one true error reported twice reads as two defects.
    if reject {
      return Err(ParseError::with(
        Code::PatternUntil,
        "`until` cannot take a pattern test: it lowers to `while !(…)`, and a pattern \
         test cannot be negated. Write `while` with the opposite pattern, or a `loop` \
         with a `break`",
        keyword_span,
      ));
    }
    return Ok(Some(StmtKind::While {
      condition,
      negated,
      body,
      label,
    }));
  }

  if p.check(&TokenKind::Kw(Keyword::For)) {
    p.advance();
    let binder = p.ident("expected a name after `for`")?;
    // Doc 30 §3.2: Rust's own `in`, which doc 18 §3's table had deleted in
    // favour of `<-` and which costs nothing to restore. `<-` now means
    // `await` and nothing else.
    p.expect(
      &TokenKind::Kw(Keyword::In),
      "expected `in` after the binder — doc 30 §3.2 writes a `for` with Rust's own word",
    )?;
    let source = parse_expr(p, 0)?;
    let body = p.statement_block("expected an indented loop body")?;
    return Ok(Some(StmtKind::For {
      binder,
      source,
      body,
      label,
    }));
  }

  if p.check(&TokenKind::Kw(Keyword::Loop)) {
    p.advance();
    let body = p.statement_block("expected an indented loop body")?;
    return Ok(Some(StmtKind::Loop { body, label }));
  }

  // A label with no loop after it is a mistake worth reporting here: the
  // alternative is rewinding and letting `'name` be read as an expression,
  // which it is not.
  if let Some(name) = label {
    return Err(ParseError::with(
      Code::ExpectedToken,
      format!("expected `while`, `until`, `for` or `loop` after the label `'{name}`"),
      p.peek_span(),
    ));
  }

  if p.check(&TokenKind::Kw(Keyword::Break)) {
    p.advance();
    let label = match &p.peek().kind {
      TokenKind::Label(name) => {
        let name = name.clone();
        p.advance();
        Some(name)
      }
      _ => None,
    };
    // A value, if the line has not ended. `break` and `break v` differ by
    // whether anything follows, which is the same test the `return` statement
    // plan 01 shipped already makes.
    let value = if p.at_statement_end() {
      None
    } else {
      Some(parse_expr(p, 0)?)
    };
    return Ok(Some(StmtKind::Break { label, value }));
  }

  if p.check(&TokenKind::Kw(Keyword::Continue)) {
    p.advance();
    let label = match &p.peek().kind {
      TokenKind::Label(name) => {
        let name = name.clone();
        p.advance();
        Some(name)
      }
      _ => None,
    };
    return Ok(Some(StmtKind::Continue { label }));
  }

  if p.check(&TokenKind::Kw(Keyword::If)) {
    return p.if_statement().map(Some);
  }

  // Doc 03 §7.1's `guard`, with doc 18 §3.1's operator. The keyword has had a
  // lexer entry since plan 02 and no parser until plan 22.
  if p.check(&TokenKind::Kw(Keyword::Guard)) {
    p.advance();
    let pattern = pattern_alt(p)?;
    p.expect(
      &TokenKind::DrawArrow,
      "expected `<-` after a `guard` pattern — doc 18 §3.1 gives `guard` the draw arrow",
    )?;
    let source = parse_expr(p, 0)?;
    p.expect(
      &TokenKind::Kw(Keyword::Else),
      "expected `else` — a `guard` without one is a binding, and let-else requires it",
    )?;
    let otherwise = p.statement_block("expected an indented `else` body after `guard`")?;
    return Ok(Some(StmtKind::Guard {
      pattern,
      source,
      otherwise,
    }));
  }

  Ok(None)
}
