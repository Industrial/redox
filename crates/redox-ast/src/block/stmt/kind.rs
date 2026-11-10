use crate::{Binding, Expr};

/// What a [`crate::Stmt`] is.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added bindings and `return`;
/// control flow is plan 20.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
  Expr(Expr),
  Binding(Binding),
  /// `return` with an optional value.
  Return(Option<Expr>),
  /// `name <- expr`, doc 18 §3.1's draw arrow in statement position.
  ///
  /// The construct doc 03 §7.6 spelled `use name <- expr`. Its lowering is a
  /// **callback**: `expr` is called with a closure whose parameter is `name`
  /// and whose body is **every statement after this one**. So a draw arrow is
  /// the one statement whose meaning depends on its successors, which is why
  /// the emitter handles a block rather than a statement at a time.
  ///
  /// ```text
  /// conn <- pool.with_connection      pool.with_connection(|conn| {
  /// print conn                   →       println!("{}", conn);
  ///                                  })
  /// ```
  Draw {
    name: crate::Ident,
    /// The callback-taking expression.
    value: Expr,
  },
  /// `user <- fetch id` — doc 30's await-bind.
  ///
  /// ```text
  /// user <- fetch id          let user = fetch(id).await;
  /// <- flush ()           →   flush().await;
  /// (a, b) <- load id         let (a, b) = load(id).await;
  /// ```
  ///
  /// **The only thing `<-` means.** Haskell's monadic bind for the monad
  /// Redox has: `do` binds an effect's result to a name and this binds a
  /// future's. Doc 30 §1.1 is the argument; doc 18 §3.1's four positions were
  /// given other spellings to free the token for it.
  ///
  /// `pattern` is `None` for the effect-only form — `<- flush ()` — which
  /// binds nothing. An `Option` rather than a `Pattern::Wildcard`, because
  /// *there is no binder* and *there is one and it discards* lower
  /// differently: the first emits `f().await;` and the second would emit
  /// `let _ = f().await;`.
  ///
  /// Legal only inside an `async` function or block, and **rustc enforces
  /// that** — doc 03 §14.3 provides no executor, so `async` passes through and
  /// the error for an await outside one is rustc's, landed on the `.sdx` by
  /// plan 07's remapping.
  Await {
    pattern: Option<crate::Pattern>,
    value: Expr,
  },
  /// `counter += 1` — doc 01 §4.3's compound assignment, plan 20.
  ///
  /// Assignment to a place — `counter := 2` and `counter += 1`, one node.
  ///
  /// # Why `:=` exists at all
  ///
  /// `=` is a **binding**. At statement level `x = v` introduces a name, and
  /// telling a rebinding from a binding by looking at it needs the scope
  /// tracking AGENTS.md forbids Redox growing. Plan 20 therefore shipped the
  /// five compound operators and no more, and disclosed the consequence
  /// instead of designing around it (§10.2): *"a loop can advance a numeric
  /// accumulator and cannot rebind anything else."*
  ///
  /// Plan 54 retires that limit with a second token rather than with
  /// analysis. `:=` is unambiguous wherever it appears, so the two meanings
  /// stop competing for one spelling and nothing has to be inferred.
  ///
  /// # Why one node and not two
  ///
  /// `a += b` *is* `a = a + b`, so a plain assignment and a compound one are
  /// one construct with a modifier. Spelling that as `Option<BinaryOp>`
  /// rather than as a second `StmtKind` variant is also what keeps this
  /// change small: `emit/stmt.rs`, `redox-fmt`'s `kind_text.rs` and two test
  /// matches are all exhaustive with no wildcard — *"a new `StmtKind` must
  /// fail to compile here"* — and none of them gains an arm.
  ///
  /// # The target is an expression, and rustc judges it
  ///
  /// Not a name: doc 18 §2.5's example assigns to `self.value`, and `v[0]`,
  /// `*r` and `(a, b)` are all reachable through the same general `Expr`.
  /// That last one is destructuring assignment, stable Rust since 1.59, and
  /// it falls out for free — plan 54 §6 declined it and §10 amends that,
  /// because refusing it would mean Redox deciding what is a place
  /// expression. `(1 + 2) := 3` is rustc's `E0070`, and *"every check rustc
  /// can make, rustc makes"* is the rule that settles which side of the line
  /// that sits on.
  Assign {
    target: Expr,
    /// `Some(op)` is `+=` and the other four; `None` is a plain `:=`.
    ///
    /// One of `Add`, `Sub`, `Mul`, `Div`, `Rem`. Reusing `BinaryOp` rather
    /// than a second enum keeps the spelling in one place — the operator's
    /// Rust text plus `=`.
    op: Option<crate::BinaryOp>,
    value: Expr,
  },
  /// `guard P = e else <diverging>` — doc 03 §7.1's early-exit binding,
  /// plan 22.
  ///
  /// ```text
  /// guard Ok raw = read p else        let Ok(raw) = read(p) else {
  ///   return Err Error::Unreadable        return Err(Error::Unreadable)
  ///                                   };
  /// ```
  ///
  /// Doc 18 §3.1 kept the keyword — *"`let`-`else` has no Haskell analogue and
  /// no other spelling"* — and gave it `<-`. Doc 30 §3.1 gives it `=`, which
  /// is what the lowering reads anyway: Rust's let-else without the `let`.
  ///
  /// The else branch must **diverge**, and Redox does not check that: it is a
  /// check rustc can make, so rustc makes it, and plan 07's remapping reports
  /// it against the `guard` line. AGENTS.md's rule, applied.
  Guard {
    pattern: crate::Pattern,
    source: Expr,
    otherwise: crate::Block,
  },
  /// `if c` with an indented block — doc 01 §8, plan 20.
  ///
  /// The **statement** form. `if c then a else b` is an expression and lives
  /// in `Expr::If`; the two are told apart by whether `then` follows the
  /// condition, which doc 18 §7 arrives the keyword for.
  If {
    condition: crate::Condition,
    then_block: crate::Block,
    /// `else` or `else if`, where the second is an `else` whose block holds
    /// one `If` statement — which is how Rust reads it too.
    otherwise: Option<crate::Block>,
  },
  /// `while c` and `until c` — doc 03 §7.7, plan 20.
  ///
  /// One node with a flag, because `until` is `while` with a negation and
  /// nothing else. Plan 16 made the same call for `const` and `static`.
  While {
    condition: crate::Condition,
    /// `until` rather than `while`. The emitted condition is **parenthesised**
    /// when set: `until a && b` is `while !(a && b)`, and `while !a && b`
    /// would be a different loop that differs on the second iteration and
    /// nowhere else — doc 02 §5.2's worst failure mode.
    negated: bool,
    body: crate::Block,
    label: Option<String>,
  },
  /// `for x <- xs` — doc 18 §3's fourth position for `<-`, plan 20.
  For {
    binder: crate::Ident,
    source: Expr,
    body: crate::Block,
    label: Option<String>,
  },
  /// `loop` — doc 03 §7.7, *"required by the break-with-value idiom"*.
  Loop {
    body: crate::Block,
    label: Option<String>,
  },
  /// `break`, `break v`, `break 'l`, `break 'l v` — doc 03 §7.2, plan 20.
  Break {
    label: Option<String>,
    value: Option<Expr>,
  },
  /// `continue` and `continue 'l`.
  Continue {
    label: Option<String>,
  },
  /// `assert e`, doc 03 §12.6's assertion spelling.
  ///
  /// A statement rather than a call, because `assert` is a reserved word —
  /// and because the lowering reads the operator the author wrote: `==`
  /// becomes `assert_eq!` and `!=` becomes `assert_ne!`, which print both
  /// values when they differ where `assert!` prints only the expression's
  /// text.
  Assert(Expr),
}
