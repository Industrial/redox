pub(crate) mod binary_op;
pub(crate) mod case_arm;
pub(crate) mod field_init;
pub(crate) mod ident;
pub(crate) mod section;
pub(crate) mod type_expr;
pub(crate) mod unary_op;

use crate::expr::case_arm::CaseArm;
use crate::expr::field_init::FieldInit;
use crate::expr::section::Section;
use crate::{BinaryOp, Ident, TypeExpr, UnaryOp};
use redox_lexer::Span;

/// An expression.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) grew this from plan 01's three
/// forms to the full expression surface, parsed by a Pratt loop over Rust's
/// precedence table — doc 02 §5.2 fixes the table as Rust's unchanged, because
/// an expression that reads like Rust and binds differently is the worst
/// available failure mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
  /// A call. Plan 19 generalises the parenless form to doc 03 §5.1's full
  /// grammar; until then only the parenthesized form and plan 01's
  /// single-argument statement-initial form are produced.
  Call {
    callee: Box<Expr>,
    args: Vec<Expr>,
    span: Span,
  },
  /// `recv.name(args)`
  MethodCall {
    recv: Box<Expr>,
    name: Ident,
    args: Vec<Expr>,
    span: Span,
  },
  /// `recv.name`
  Field {
    recv: Box<Expr>,
    name: Ident,
    span: Span,
  },
  /// `recv[index]`
  Index {
    recv: Box<Expr>,
    index: Box<Expr>,
    span: Span,
  },
  Binary {
    op: BinaryOp,
    lhs: Box<Expr>,
    rhs: Box<Expr>,
    span: Span,
  },
  Unary {
    op: UnaryOp,
    operand: Box<Expr>,
    span: Span,
  },
  /// `(inner)` — kept in the tree so the emitter reproduces the grouping the
  /// author wrote rather than re-deriving it from precedence.
  Paren {
    inner: Box<Expr>,
    span: Span,
  },
  /// `a::b::c` — a path, carried verbatim (doc 03 §13.1).
  Path {
    text: String,
    span: Span,
  },
  /// `\x -> e`, doc 18 §4's lambda.
  ///
  /// Lowers to a Rust closure: `|x| e`, with `move` and per-parameter types
  /// carried through. Doc 18 §8 declines currying, so the parameters are a
  /// list rather than nested single-parameter lambdas — `\x y -> e` is one
  /// closure of two arguments, matching how application works.
  Lambda {
    params: Vec<crate::Param>,
    body: Box<Expr>,
    is_move: bool,
    span: Span,
  },
  /// `e :: T`, doc 18 §6's ascription.
  ///
  /// Lowers to `{ let __rdx_t: T = e; __rdx_t }` — a block, because Rust has
  /// no expression-level ascription and a `let` is the only way to state a
  /// type without naming a path. One of doc 18's three new lowerings, and the
  /// one that replaces doc 03's `into` clause and most turbofish needs.
  ///
  /// Type-free by construction: the type is **written**, never inferred.
  Ascribe {
    inner: Box<Expr>,
    ty: TypeExpr,
    span: Span,
  },
  /// A string literal, stored **raw, including its delimiters**. Interpolation
  /// is plan 23; the lexer already produces the parts.
  StrLit {
    value: String,
    span: Span,
  },
  /// `Point { x = 10 }` and `p { x = 99 }`, doc 18 §2.4's record literal.
  ///
  /// One node for both construction and update, because the source is one
  /// form: `head { fields }`. Which it *means* is decided by the case of the
  /// head — doc 18 §5.3 makes case meaningful, and §2.4 says that is "why the
  /// update form is admissible at all". An uppercase head names a
  /// constructor and builds; anything else names a value and updates.
  ///
  /// Update is one of doc 18 §6's disclosed lowerings:
  /// `{ let mut __rdx_r = p; __rdx_r.x = 99; __rdx_r }`. Rust's own
  /// `Point { x: 99, ..p }` would need the type's *path*, which the source
  /// does not give — only the value's name.
  Record {
    head: Box<Expr>,
    fields: Vec<FieldInit>,
    span: Span,
  },
  /// `()` — the unit value.
  ///
  /// Doc 18 §7 retires doc 03 §5.2's `!` zero-argument suffix in favour of
  /// `f ()`, which makes the unit expression load-bearing rather than
  /// decorative: without it a no-argument call has no spelling at all, since
  /// juxtaposition needs something to juxtapose.
  Unit {
    span: Span,
  },
  /// A section — an operator with one side supplied, doc 18 §4.
  ///
  /// Always a one-parameter closure over the reserved `__rdx_1`, so a section
  /// is a value and needs no special call site.
  Section {
    section: Section,
    span: Span,
  },
  /// `case e of` with `->` arms, doc 18 §2.3.
  ///
  /// Lowers to Rust's `match`, which it already was in all but spelling: the
  /// arms, the guards and the alternation are Rust's, and what changes is the
  /// arrow. `switch` leaves the language with this.
  Case {
    scrutinee: Box<Expr>,
    arms: Vec<CaseArm>,
    span: Span,
  },
  /// An integer or float literal, carried verbatim with any suffix.
  Num {
    text: String,
    span: Span,
  },
  Bool {
    value: bool,
    span: Span,
  },
  Char {
    value: char,
    span: Span,
  },
  /// `@field` → `self.field` (doc 01 §16.1).
  SelfField {
    name: Ident,
    span: Span,
  },
  /// `@@name` → `Self::name` (doc 01 §16.2).
  SelfType {
    name: Ident,
    span: Span,
  },
  Ident(Ident),
}

impl Expr {
  pub fn span(&self) -> Span {
    match self {
      Expr::Call { span, .. }
      | Expr::MethodCall { span, .. }
      | Expr::Field { span, .. }
      | Expr::Index { span, .. }
      | Expr::Binary { span, .. }
      | Expr::Unary { span, .. }
      | Expr::Paren { span, .. }
      | Expr::Path { span, .. }
      | Expr::Lambda { span, .. }
      | Expr::Ascribe { span, .. }
      | Expr::StrLit { span, .. }
      | Expr::Case { span, .. }
      | Expr::Record { span, .. }
      | Expr::Section { span, .. }
      | Expr::Unit { span }
      | Expr::Num { span, .. }
      | Expr::Bool { span, .. }
      | Expr::Char { span, .. }
      | Expr::SelfField { span, .. }
      | Expr::SelfType { span, .. } => *span,
      Expr::Ident(i) => i.span,
    }
  }
}
