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
//! hold a `StmtKind` or a `Binding`; an `Expr` holds an `Ident`, its operators
//! and a `TypeExpr`. Per doc 06 §3 rule 1 this crate holds no decisions, only
//! types.

mod block;
mod expr;
mod program;

pub use block::Block;
pub use block::stmt::Stmt;
pub use block::stmt::binding::Binding;
pub use block::stmt::kind::StmtKind;
pub use expr::Expr;
pub use expr::binary_op::BinaryOp;
pub use expr::case_arm::CaseArm;
pub use expr::field_init::FieldInit;
pub use expr::ident::Ident;
pub use expr::qualifier::Qualifier;
pub use expr::section::Section;
pub use expr::type_expr::TypeExpr;
pub use expr::unary_op::UnaryOp;
pub use program::Program;
pub use program::item::Item;
pub use program::item::class_decl::ClassDecl;
pub use program::item::class_member::ClassMember;
pub use program::item::clause::Clause;
pub use program::item::clause_body::ClauseBody;
pub use program::item::constraint::Constraint;
pub use program::item::constructor::Constructor;
pub use program::item::constructor_fields::ConstructorFields;
pub use program::item::data_decl::DataDecl;
pub use program::item::error_variant::ErrorVariant;
pub use program::item::errors_decl::ErrorsDecl;
pub use program::item::field_decl::FieldDecl;
pub use program::item::fn_decl::FnDecl;
pub use program::item::guard::Guard;
pub use program::item::instance_decl::InstanceDecl;
pub use program::item::param::Param;
pub use program::item::pattern::Pattern;
pub use program::item::signature::Signature;
pub use program::item::test_decl::TestDecl;
pub use program::item::type_alias::TypeAlias;
pub use program::item::type_app::TypeApp;
pub use program::item::visibility::Visibility;

// Re-exported so downstream crates naming a span or a trivium need not add a
// second dependency edge onto `redox-lexer` (doc 06 §5).
pub use redox_lexer::{Span, Trivia, TriviaKind};
