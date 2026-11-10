use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::method_context::MethodContext;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::receiver::receiver;
use crate::parse::parser::substitute_var::substitute_var;
use crate::parse::parser::type_stop::TypeStop;
use crate::parse::parser::visibility::visibility;
use redox_ast::{ClassMember, Ident};
use redox_diag::Code;
use redox_lexer::{Keyword, TokenKind};

/// One member of a `class` or `instance` body — doc 18 §2.5.
///
/// ```text
/// type Unit               an associated type
/// const SIDES : u32      an associated constant
/// area : &self -> f64    a method signature, with no body
/// describe : &self -> String
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
  // **Peeked before anything is consumed**, which plan 68 found the hard way:
  // this was read further down, after `visibility(p)` had taken the `export`
  // the trivia was attached to, so every doc comment on a member with a
  // visibility marker was silently discarded. `FnDecl` has carried a
  // `leading` field all along and `emit_member` has always printed it — the
  // capture was one token too late.
  //
  // The same shape as plan 68's fix to `constructor`, where the doc comment
  // sits above the `=` or `|` rather than above the name. Peek first.
  let leading = p.peek_leading();
  // **And the span, for the same reason.** `start` used to be read further
  // down, after `visibility(p)` had taken the `export` — which the emitter
  // never minded, because it prints `decl.visibility`, and which broke the
  // *formatter*: `signature_line` rebuilds a signature from the source slice
  // at `decl.span.0`, so a span beginning at the name produced a signature
  // with no visibility, and `redox fmt` wrote that back as canonical. Every
  // member with a marker lost it — `pub fn` became `fn`.
  //
  // Plan 68 moved `peek_leading` above `visibility` for the trivia and left
  // this one behind. Plan 73 found it by putting `redox fmt --check` in the
  // gate, which is the only thing that reads these members back.
  let start = p.peek_span().0;

  // A trait's members take their visibility from the trait, and an inherent
  // `impl`'s may each have their own — so the marker is read here and rustc
  // rejects it where it is meaningless, which is the division of labour
  // AGENTS.md states.
  let member_visibility = visibility(p)?;

  if p.check(&TokenKind::Kw(Keyword::Type)) {
    p.advance();
    let name = p.ident("expected a name after `type`")?;
    p.newline_with_trailing("expected a line break after the associated type")?;
    return Ok(ClassMember::AssocType(name));
  }

  if p.check(&TokenKind::Kw(Keyword::Const)) {
    p.advance();
    let name = p.ident("expected a name after `const`")?;
    p.expect(&TokenKind::Colon, "expected `:` after the constant's name")?;
    let ty = capture_type(p, "expected the constant's type", TypeStop::Greedy)?;
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

  // The head's variables belong to the enclosing declaration, not to this
  // method, so they are kept out of the method's generic list.
  let reserved = ctx.reserved();

  // Plan 63. An `instance` member may omit its signature, and the class
  // supplies it. The discriminator is the one doc 18 §2.3 already uses: a
  // name followed by `:` opens a signature line, and a name not followed by
  // one opens an equation.
  //
  // Leaf 0's blocker is why this is read here rather than inside
  // `Parser::function`: that function's clause loop treats any `Ident` not
  // followed by `:` as a *continuation equation of the member it is already
  // reading*, so an elided member collides with the multi-clause grammar and
  // the name-mismatch guard fires — `RDX0103: this equation defines `x`, but
  // the signature above it is for `y``. Reading the name here, and branching
  // before the loop starts, is what keeps the two apart.
  // `start` is **not** taken here: it is captured at the top of this function,
  // before `visibility(p)`, so the span covers the member's `export`. The
  // comment there says what went wrong when it was not.
  // The name is **peeked**, not consumed, and that is load-bearing rather
  // than fussy: `Parser::equations`'s loop reads the name once per equation,
  // because doc 18 §2.3 makes every equation repeat it. A member with a
  // signature line spells its name twice — once on the line, once on the
  // equation — and an elided one spells it once. So consuming it here would
  // leave the loop looking at `self` where it expects `cents_of`, which is
  // exactly the `RDX0103: expected a member's name` the first attempt earned.
  let TokenKind::Ident(peeked) = &p.peek().kind else {
    return Err(ParseError::with(
      Code::ExpectedName,
      "expected a member's name".to_string(),
      p.peek_span(),
    ));
  };
  let name = Ident {
    text: peeked.clone(),
    span: p.peek_span(),
  };
  let has_signature = p.peek_at(1).is_some_and(|t| t.kind == TokenKind::Colon);
  let head = if has_signature {
    let name = p.ident("expected a member's name")?;
    let head = p.signature_head(&name)?;
    // Collect and never transform — plan 61's rule. A `class` records what it
    // states so an `instance` can borrow it; nothing is rewritten either way.
    if let Some(class) = &ctx.recording {
      p.record_head(class, &name.text, &head);
    }
    head
  } else {
    let borrowed = ctx
      .implementing
      .as_deref()
      .and_then(|class| p.borrowed_head(class, &name.text));
    match borrowed {
      Some(head) => head,
      // Nothing to borrow: no class, a class this file has not declared, or a
      // class declared *after* this instance. §10 states that last one as the
      // row's narrowing rather than hiding it, and the diagnostic says which
      // of the three it was by naming what it looked for.
      None => {
        return Err(ParseError::with(
          Code::ExpectedToken,
          match ctx.implementing.as_deref() {
            Some(class) => format!(
              "`{}` has no signature here, and `{class}` declares none for it earlier in this file — write the signature, or declare the class above the instance",
              name.text
            ),
            None => format!(
              "`{}` needs a signature: an inherent `instance` has no class to take one from",
              name.text
            ),
          },
          name.span,
        ));
      }
    }
  };
  let mut decl = p.equations(name, leading, start, &head, &reserved, true)?;
  decl.visibility = member_visibility;
  receiver(&mut decl.sig);
  for (var, subject) in &ctx.vars {
    substitute_var(&mut decl.sig, var, subject);
  }
  Ok(ClassMember::Method(decl))
}
