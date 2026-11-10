// **Plan 55 §1.6's unit tail is `clippy::no_effect`.**
//
// `RDX0113` requires a `()`-returning body to end in a written `()`, so that
// a unit return is visible rather than inferred, and the `();` that generates
// is what clippy calls *"statement with no effect"*. Under `-D warnings` the
// two rules cannot both be satisfied by any Redox a person could write: the
// lint fires on the thing the language insists on.
//
// Plan 68 found it and owes the fix to a successor, where it is small and
// needs no type knowledge: `emit_clauses` can **drop a trailing `()`
// statement** from a `()`-returning body, because it contributes nothing and
// the signature already says the function returns unit. §1.6's rule governs
// what the author writes, not what the emitter passes through. It would move
// every golden, which is why it is a row rather than a line here.
#![allow(clippy::no_effect)]
// **`matches!` is unreachable from Redox, so a `case` ladder is the spelling.**
//
// Plan 64 made macros callable as `name! \-> e`, and §6 of that plan
// discloses what the shape cannot carry: `matches!`'s second argument is a
// **pattern**, not an expression. Six methods in this crate ask *is this one
// of these variants*, and in Rust they were `matches!`; in Redox they are a
// `case` with a `true` arm and a `_ -> false`.
//
// `clippy::match_like_matches_macro` then fires on every one of them, asking
// for a macro this language cannot call. The allow is a statement of that
// fact rather than a suppression of a defect — plan 68 found it, and the only
// thing that would retire it is a row making `matches!` reachable.
#![allow(clippy::match_like_matches_macro)]

//! `redox-ast` — syntax tree types. **No logic.**
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`) leaf 3 established the shape;
//! plan 03 (`docs/08 - 03 - Parser Core.md`) grew it to the expression,
//! signature and binding surface.
//!
//! Every node carries a byte [`Span`], and every node that can own source
//! lines carries [`Trivia`]. Both are built in from the start on evidence:
//! Emerald's plan 22 found `TypeExpr` carried no span at all, so every
//! diagnostic touching a type hardcoded `(0, 0)` and reported line 1 column 1
//! for an error on line 6 — discovered twenty-one plans after its diagnostics
//! plan shipped, by which point the full fix was judged too large.
//!
//! Deliberately not generic over a span type and deliberately not
//! arena-allocated (doc 01 §30's minimality constraint).
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`. The module tree is
//! the grammar's containment hierarchy: a `Program` holds `Item`s which hold a
//! `FnDecl` with its `Signature` and `Param`s; a `Block` holds `Stmt`s which
//! hold an `Expr`; an `Expr` holds an `Ident`, its operators and a
//! `TypeExpr`. Per doc 06 §3 rule 1 this crate holds no decisions, only types.
//!
//! **There is one node kind for a body position.** Plan 55 leaf 1 deleted
//! `StmtKind` and moved its forms into [`Expr`], so a block element and a
//! subexpression are the same type and the statement/expression fork is gone
//! from the tree. [`Stmt`] survives as the trivia carrier and records why.

mod block;

mod condition;

mod expr;

mod program;

mod where_cycle;

mod where_edges;

pub use block::Block;

pub use block::stmt::Stmt;

pub use block::stmt::binding::Binding;

pub use condition::Condition;

pub use condition::operand::ConditionOperand;

pub use expr::Expr;

pub use expr::binary_op::BinaryOp;

pub use expr::case_arm::CaseArm;

pub use expr::field_init::FieldInit;

pub use expr::ident::Ident;

pub use expr::qualifier::Qualifier;

pub use expr::section::Section;

pub use expr::str_interp_part::StrInterpPart;

pub use expr::type_expr::TypeExpr;

pub use expr::unary_op::UnaryOp;

pub use program::Program;

pub use program::item::Item;

pub use program::item::class_decl::ClassDecl;

pub use program::item::class_member::ClassMember;

pub use program::item::clause::Clause;

pub use program::item::clause_body::ClauseBody;

pub use program::item::const_decl::ConstDecl;

pub use program::item::constraint::Constraint;

pub use program::item::constructor::Constructor;

pub use program::item::constructor_fields::ConstructorFields;

pub use program::item::data_decl::DataDecl;

pub use program::item::error_variant::ErrorVariant;

pub use program::item::errors_decl::ErrorsDecl;

pub use program::item::extern_block::ExternBlock;

pub use program::item::field_decl::FieldDecl;

pub use program::item::fn_decl::FnDecl;

pub use program::item::foreign_fn::ForeignFn;

pub use program::item::guard::Guard;

pub use program::item::import_decl::ImportDecl;

pub use program::item::inline_rust::InlineRust;

pub use program::item::instance_decl::InstanceDecl;

pub use program::item::leading::leading;

pub use program::item::module_decl::ModuleDecl;

pub use program::item::param::Param;

pub use program::item::pattern::Pattern;

pub use program::item::primitive_type::primitive_type;

pub use program::item::record_pattern_field::RecordPatternField;

pub use program::item::signature::Signature;

pub use program::item::test_decl::TestDecl;

pub use program::item::type_alias::TypeAlias;

pub use program::item::type_app::TypeApp;

pub use program::item::visibility::Visibility;

// Re-exported so downstream crates naming a span or a trivium need not add a
// second dependency edge onto `redox-lexer` (doc 06 §5).
pub use crate::where_cycle::where_cycle;

pub use crate::where_edges::where_edges;

pub use redox_lexer::{Span, Trivia, TriviaKind};
