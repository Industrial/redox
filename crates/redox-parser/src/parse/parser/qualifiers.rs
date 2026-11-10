use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::parse_expr::parse_expr;
use redox_ast::Qualifier;
use redox_lexer::{Keyword, TokenKind};

/// The comma-separated qualifiers right of a comprehension's bar — doc 18
/// §3.2.
///
/// ```text
/// [ x * y | x in 1..3, y in 1..3, x != y ]
/// ```
///
/// A generator is `name <- source`, recognised by two tokens of lookahead —
/// an identifier then `<-`, which is unambiguous because `<-` is a single
/// token. Anything else is a guard. The same test `try_draw` uses for a
/// statement, and for the same reason: doc 18 §3's four positions for `<-`
/// are grammatically disjoint, so no backtracking is needed to tell them
/// apart.
pub(crate) fn qualifiers(p: &mut Parser) -> Result<Vec<Qualifier>, ParseError> {
  let mut found = Vec::new();
  loop {
    // Doc 30 §3.2: the generator takes `in`, not `<-`. `<-` means `await` and
    // nothing else, so the token doc 18 §3.2 borrowed from Haskell's
    // comprehension is given up — §4.2 records that as the real loss, taken to
    // free the token for the position where it is more faithful.
    let is_draw = matches!(p.peek().kind, TokenKind::Ident(_))
      && p
        .peek_at(1)
        .is_some_and(|t| t.kind == TokenKind::Kw(Keyword::In));

    if is_draw {
      let name = p.ident("expected a name before `in`")?;
      p.advance();
      let source = parse_expr(p, 0)?;
      found.push(Qualifier::Draw { name, source });
    } else {
      found.push(Qualifier::Guard(parse_expr(p, 0)?));
    }

    p.skip_layout();
    if !p.check(&TokenKind::Comma) {
      break;
    }
    p.advance();
    p.skip_layout();
  }
  Ok(found)
}
