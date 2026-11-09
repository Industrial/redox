pub(crate) mod binary_op;
pub(crate) mod case_arm;
pub(crate) mod field_init;
pub(crate) mod ident;
pub(crate) mod qualifier;
pub(crate) mod section;
pub(crate) mod str_interp_part;
pub(crate) mod type_expr;
pub(crate) mod unary_op;

use crate::expr::case_arm::CaseArm;
use crate::expr::field_init::FieldInit;
use crate::expr::qualifier::Qualifier;
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
  /// `user?.profile?.name` — doc 01 §13.1's optional chain, plan 28.
  ///
  /// The head and a run of `?.`-joined **field names**. Doc 03 §12.3's
  /// grammar chains field accesses; a method on the end of a chain is an
  /// ordinary postfix on the chain's result, so it is not here.
  ///
  /// Lowered via doc 02 §6.3's closure — `?` inside a closure returning
  /// `Option` — which hands the whole `and_then`/`map` unification to rustc.
  /// Every link takes `as_ref()`, because §12.3 resolves doc 02 §13's
  /// question 7 outright: **the chain borrows and never clones.**
  OptionalChain {
    head: Box<Expr>,
    segments: Vec<String>,
    span: Span,
  },
  /// `try` with an indented block — doc 03 §12.4, plan 28.
  ///
  /// A block in which `?` returns from the **block** rather than the
  /// enclosing function, lowered to an immediately-invoked closure. Rust has
  /// wanted this for years and has it only unstably, so this is the one row
  /// that makes an unavailable Rust construct available rather than an
  /// available one nicer to spell.
  ///
  /// Distinct from plan 20's `Do` by exactly the `(|| … )()` wrapper: a `try`
  /// lowered to a bare block compiles, and its `?` then returns from the
  /// function. That is why plan 28 §3 proves it by running.
  TryBlock {
    body: crate::Block,
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
  /// A string literal with **no** interpolation, stored raw, including its
  /// delimiters — which is what plan 02 carried `raw` for, and why an
  /// interpolated string is a different node rather than a flag on this one.
  ///
  /// A `"""` block is **not** stored raw: Rust has no triple-quoted string,
  /// so the lexer's dedented value arrives as a [`StrLit`](Self::StrLit)
  /// holding an ordinary escaped Rust string. Plan 23 §1.3.
  StrLit {
    value: String,
    span: Span,
  },
  /// An interpolated string — doc 03 §11.1, plan 23.
  ///
  /// ```text
  /// "hello #{name}"           format!("hello {}", name)
  /// "total: #{amount:.2}"  →  format!("total: {:.2}", amount)
  /// ```
  ///
  /// Separate from `StrLit` so that every string without an interpolation
  /// takes exactly the path it took before this plan — which plan 23's
  /// criterion 6 asserts, because one node for both would have made every
  /// existing golden a diff.
  StrInterp {
    parts: Vec<crate::StrInterpPart>,
    span: Span,
  },
  /// `[0u8; 1024]` — Rust's array-repeat expression, doc 03 §10.4 and §18,
  /// plan 29.
  ///
  /// **Pure passthrough**, and a node of its own rather than a flag on
  /// [`List`](Self::List) for a reason plan 29 §1.3 states: a list literal
  /// lowers to `vec![…]`, so a list is growable by default, and a repeat
  /// expression is a **fixed-size array** — which is the only reason to write
  /// one. Emitting `vec![0u8; 1024]` would compile, run, and silently
  /// allocate on the heap where the author asked for a stack array.
  ///
  /// The two share a bracket and nothing else; the `;` tells them apart.
  ArrayRepeat {
    value: Box<Expr>,
    count: Box<Expr>,
    span: Span,
  },
  /// A literal passed through **verbatim and never used as a format string** —
  /// `r"…"`, `r#"…"#`, `b'x'`, `b"…"`, `c"…"`. Doc 03 §11.2, plan 23.
  ///
  /// §11.2 calls all four *"pure passthrough"*, and the reason they are one
  /// node is that nothing downstream needs to tell them apart: none may be
  /// escaped, brace-doubled or treated as a template. Keeping them out of
  /// `StrLit` is what stops `print b"x"` emitting `println!(b"x")`.
  ///
  /// §11.3 declines regex literals **on condition** that `r"…"` exists, so
  /// this node is what makes that decline honest.
  RawLit {
    text: String,
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
  /// `[a, b, c]` — a list literal, which is Rust's array literal unchanged.
  ///
  /// A spelling rather than a lowering. It is here because doc 18 §3.2's
  /// comprehension shares its brackets, so the two are told apart by what
  /// follows the first expression — a `|` opens a comprehension, a `,` or a
  /// `]` closes a literal — and reading one without the other would have
  /// left `[` ambiguous.
  List {
    items: Vec<Expr>,
    span: Span,
  },
  /// `[ e | x <- xs, cond ]` — doc 18 §3.2's comprehension.
  Comprehension {
    element: Box<Expr>,
    qualifiers: Vec<Qualifier>,
    span: Span,
  },
  /// `if c then a else b` — doc 01 §8's conditional **expression**, plan 20.
  ///
  /// Doc 18 §7 arrives `then` for exactly this construct and nothing else
  /// uses the word, which is why two surface forms exist for one Rust
  /// construct: the alternative is a reserved keyword with no use.
  ///
  /// The `else` is not optional. Rust's `if` without one has type `()`, so a
  /// conditional *expression* that omitted it could only ever be a statement
  /// — and the statement form is `StmtKind::If`.
  If {
    condition: Box<crate::Condition>,
    then: Box<Expr>,
    otherwise: Box<Expr>,
    span: Span,
  },
  /// `do` and `do 'label` — doc 03 §7.2's block expression, plan 20.
  ///
  /// An expression, because §7.2's own example assigns one to a binding. The
  /// labelled form is Rust's labelled block, native since 1.65, which is what
  /// *"permitting early exit with a value"* needs.
  Do {
    label: Option<String>,
    body: crate::Block,
    span: Span,
  },
  /// `e.await` — doc 03 §14.3's postfix await, plan 15.
  ///
  /// **A correction, not a feature.** Doc 01 §24 specified prefix `await`,
  /// which Rust does not have: it has had postfix `.await` since 1.39. §14.3
  /// gives the reason the postfix form is the right one in one sentence —
  /// *"postfix composes with method chains, with `?`, and with the receiver
  /// pipeline of §5.3; prefix requires parenthesising to do any of them."*
  ///
  /// A node of its own rather than an `Expr::Field` named `await`, because
  /// `await` is a reserved word and is not an `Ident` — a node pretending
  /// otherwise would let the formatter and the span map both read it wrongly.
  Await {
    inner: Box<Expr>,
    span: Span,
  },
  /// `e?` — Rust's try operator, unchanged.
  ///
  /// A spelling, not a lowering. It is here because doc 03 §12.5's `from`
  /// row exists to make `?` work across an error boundary, and a generated
  /// `From` impl with no `?` to use it would be a feature with no caller.
  Try {
    inner: Box<Expr>,
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
      | Expr::OptionalChain { span, .. }
      | Expr::TryBlock { span, .. }
      | Expr::Ascribe { span, .. }
      | Expr::StrLit { span, .. }
      | Expr::StrInterp { span, .. }
      | Expr::ArrayRepeat { span, .. }
      | Expr::RawLit { span, .. }
      | Expr::Case { span, .. }
      | Expr::Record { span, .. }
      | Expr::Section { span, .. }
      | Expr::Unit { span }
      | Expr::Try { span, .. }
      | Expr::Await { span, .. }
      | Expr::If { span, .. }
      | Expr::Do { span, .. }
      | Expr::List { span, .. }
      | Expr::Comprehension { span, .. }
      | Expr::Num { span, .. }
      | Expr::Bool { span, .. }
      | Expr::Char { span, .. }
      | Expr::SelfField { span, .. }
      | Expr::SelfType { span, .. } => *span,
      Expr::Ident(i) => i.span,
    }
  }
}
