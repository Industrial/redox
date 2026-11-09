use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::method_context::MethodContext;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::receiver::receiver;
use crate::parse::parser::substitute_var::substitute_var;
use redox_ast::ClassMember;
use redox_lexer::{Keyword, TokenKind};

/// One member of a `class` or `instance` body — doc 18 §2.5.
///
/// ```text
/// type Unit               an associated type
/// const SIDES :: u32      an associated constant
/// area :: &t -> f64       a method signature, with no body
/// describe :: &t -> String
/// describe self = "…"     a signature and its default body
/// ```
///
/// A method is parsed by the **same** code a top-level function is, through
/// `Parser::function`, which is why a default method gets several equations
/// and a guard ladder for free: doc 18 §2.3's clauses are not a function-only
/// construct, they are what a body is.
///
/// `ctx` carries what the enclosing declaration knows about its own subject
/// — see [`MethodContext`].
pub(crate) fn class_member(p: &mut Parser, ctx: &MethodContext) -> Result<ClassMember, ParseError> {
  if p.check(&TokenKind::Kw(Keyword::Type)) {
    p.advance();
    let name = p.ident("expected a name after `type`")?;
    p.newline_with_trailing("expected a line break after the associated type")?;
    return Ok(ClassMember::AssocType(name));
  }

  if p.check(&TokenKind::Kw(Keyword::Const)) {
    p.advance();
    let name = p.ident("expected a name after `const`")?;
    p.expect(
      &TokenKind::ColonColon,
      "expected `::` after the constant's name",
    )?;
    let ty = capture_type(p, "expected the constant's type")?;
    // A `class` declares the constant and an `instance` supplies it, so the
    // value is optional here and rustc decides which was required.
    let value = if p.check(&TokenKind::Eq) {
      p.advance();
      Some(parse_expr(p, 0)?)
    } else {
      None
    };
    p.newline_with_trailing("expected a line break after the associated constant")?;
    return Ok(ClassMember::AssocConst { name, ty, value });
  }

  let mut decl = p.function()?;
  receiver(&mut decl.sig, &ctx.receiver_names);
  if let Some(var) = &ctx.var {
    substitute_var(&mut decl.sig, var, &ctx.subject);
  }
  Ok(ClassMember::Method(decl))
}
