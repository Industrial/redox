//! `redox-ast` — syntax tree types. No logic.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 3.
//!
//! Every node carries a byte [`Span`], and every node that can own source
//! lines carries [`Trivia`]. Both are built now rather than added later, on
//! evidence: Emerald's plan 22 found `TypeExpr` carried no span at all, so
//! every diagnostic touching a type hardcoded `(0, 0)` and reported line 1
//! column 1 for an error on line 6 — discovered twenty-one plans after its
//! diagnostics plan shipped, by which point the full fix was judged too large
//! and it settled for a partial workaround.
//!
//! Deliberately not generic over a span type and deliberately not
//! arena-allocated (doc 01 §30's minimality constraint). Neither is needed
//! until a profile says otherwise.

pub use redox_lexer::{Span, Trivia, TriviaKind};

/// A whole source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
  pub items: Vec<Item>,
  pub span: Span,
  /// Trivia after the last item — a trailing comment at end of file.
  pub trailing: Vec<Trivia>,
}

/// A top-level declaration. Plan 01's slice has exactly one form; structs,
/// enums, traits and impls are plan 24 and plan 25.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
  Fn(FnDecl),
}

impl Item {
  pub fn span(&self) -> Span {
    match self {
      Item::Fn(f) => f.span,
    }
  }
}

/// `name = ->` followed by an indented block.
///
/// Parameters, type annotations and return types are plan 03; this slice's
/// grammar has a bare name and a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
  pub name: Ident,
  pub body: Block,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}

/// An indented sequence of statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
  pub stmts: Vec<Stmt>,
  pub span: Span,
}

/// A statement. Plan 01 has only the expression form; `let` bindings are
/// plan 03 and control flow is plan 20.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
  pub kind: StmtKind,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
  Expr(Expr),
}

/// An expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
  /// A call. This slice produces only the parenless, single-argument,
  /// statement-initial form; plan 19 generalises it to doc 03 §5.1's full
  /// call grammar.
  Call {
    callee: Box<Expr>,
    args: Vec<Expr>,
    span: Span,
  },
  /// A string literal, stored **raw, including its delimiters** — see
  /// `redox_lexer::TokenKind::StrLit`. Escape decoding is plan 23.
  StrLit {
    value: String,
    span: Span,
  },
  Ident(Ident),
}

impl Expr {
  pub fn span(&self) -> Span {
    match self {
      Expr::Call { span, .. } | Expr::StrLit { span, .. } => *span,
      Expr::Ident(i) => i.span,
    }
  }
}

/// An identifier and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
  pub text: String,
  pub span: Span,
}
