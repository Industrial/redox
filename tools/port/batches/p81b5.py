# -*- coding: utf-8 -*-
"""Plan 81 batch 5: the three `thread_local!` guards — and the escape hatch.

**`thread_local!` takes items, and a Redox macro argument is an expression.**
Plan 64 made macros callable with a nullary lambda per argument; §6 of that
plan disclosed that `matches!`'s *pattern* argument is out of reach, and this
is the same gap one step further out — a macro whose body is a `static`
declaration. There is no Redox spelling, so these three use doc 03 §23's
inline-Rust hatch: the `rust` keyword and a triple-quoted block.

That makes `redox-emit` the first ported crate to need the hatch, which is
exactly the property doc 02 §4.8 asks about — so it is recorded rather than
quietly used. A plain `static` will not do instead: the mode is a `Cell` and
the tables are `RefCell`s, neither of which is `Sync`, and the `thread_local!`
is there so the suite's parallel tests cannot see each other's files.

**The comments inside a hatch stay `///`, and the check learned to read
them.** A doc comment *above* the hatch item would be `unused_doc_comments`
on a macro invocation, which `-D warnings` makes an error — so they belong
inside. And a `///` inside a verbatim block has no `-- |` to come from, which
made `every_doc_comment_reaches_the_generated_rust` report a comment appearing
out of nowhere; `port.rs` now counts a verbatim block's own doc comments on
both sides. The alternative was to downgrade them to `//` to satisfy the
arithmetic, which is the test changing the code rather than reading it.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/bind_scope', '''
-- The `thread_local!` block, verbatim: a macro whose body is a `static`
-- declaration has no Redox spelling, because a macro argument is an
-- expression. Doc 03 §23's hatch, and `redox-emit` §2 records it.
rust """
  thread_local! {
    /// What `<-` means right here. [`BindMode::Await`] outside any modifier,
    /// which is the meaning doc 30 gave the token and the one `async fn` keeps.
    static MODE: Cell<BindMode> = const { Cell::new(BindMode::Await) };
  }
"""

-- | The bind mode, installed for the body of one block modifier.
-- |
-- | Plan 60 (`docs/54 - 60 - The Bind Arrow.md`) leaf 3. A guard rather than a
-- | setter, because the modes **nest**: a `try` inside an `async` and an
-- | `async` inside a `try` both have to restore the outer meaning when their
-- | body ends, and an early return must not leave the wrong one installed.
-- |
-- | `Cell` rather than `RefCell`, because the mode is `Copy` and nothing
-- | borrows it across a call. `thread_local!` for `ArityScope`'s reason: the
-- | suite's parallel tests must not see each other's blocks.
export(crate) data BindScope = BindScope {
  -- | The mode to put back when the guard drops.
  previous : BindMode,
}

instance BindScope
  -- | Install `mode` for as long as the returned guard lives.
  export(crate) enter : BindMode -> Self
  enter mode = BindScope { previous = .replace mode MODE }

  -- | What `<-` means at this point in the emission.
  export(crate) mode : BindMode
  mode = .get MODE

instance Drop BindScope
  drop : &mut self -> ()
  drop self =
    .set self.previous MODE
    ()
''')

write('emit/binding_scope', '''
-- The `thread_local!` block, verbatim — see `bind_scope` for why.
rust """
  thread_local! {
    /// The table in scope, empty outside a [`BindingScope`].
    static IN_SCOPE: RefCell<BindingTable> = RefCell::new(BindingTable::default());
  }
"""

-- | The binding table, installed for the duration of one file's emission.
-- |
-- | Plan 59 (`docs/53 - 59 - Value Bindings.md`) leaf 1, and the placement is
-- | cited rather than re-derived: the table rides the **emitter's entry** and
-- | nothing earlier, because `redox_fmt::format` consumes the same `Program`
-- | and must keep printing the author's `block` as `block`. A rewrite visible
-- | before this point would make the formatter rewrite the surface.
-- |
-- | A `thread_local!` rather than a `static`, so the suite's parallel tests
-- | cannot see each other's files; a guard rather than a setter, so an early
-- | return cannot leave a stale table behind. `ArityScope`'s shape exactly —
-- | two guards over two tables, entered side by side in `emit_with_map`.
export(crate) data BindingScope = BindingScope {
  -- | The table to put back when the guard drops.
  previous : BindingTable,
}

instance BindingScope
  -- | Install `table` for as long as the returned guard lives.
  export(crate) enter : BindingTable -> Self
  enter table =
    previous = .with_borrow_mut (\\slot -> std::mem::replace slot table) IN_SCOPE
    BindingScope { previous }

  -- | What `name` means in the file being emitted.
  export(crate) kind_of : &str -> Option<BindingKind>
  kind_of name = .with_borrow (\\table -> .kind_of name table) IN_SCOPE

instance Drop BindingScope
  drop : &mut self -> ()
  drop self =
    restored = std::mem::take (&mut self.previous)
    -- `std::mem::replace` and a discard, where the Rust wrote `*slot =
    -- restored`: a lambda body is one *expression* and `:=` is an assignment,
    -- so the assignment has no place to sit. `replace` says the same thing and
    -- is `enter`'s own idiom one function up.
    _dropped = .with_borrow_mut (\\slot -> std::mem::replace slot restored) IN_SCOPE
    ()
''')

write('emit/arity_scope', '''
-- The `thread_local!` block, verbatim — see `bind_scope` for why.
rust """
  thread_local! {
    /// The table in scope, empty outside an [`ArityScope`].
    ///
    /// Empty is the safe default and that is load-bearing: an arity nobody
    /// installed is an arity nobody knows, which is the case plan 56 §11.1
    /// settles by emitting a call.
    static IN_SCOPE: RefCell<ArityTable> = RefCell::new(ArityTable::default());
  }
"""

-- | The arity table in scope for the file being emitted.
-- |
-- | Plan 56 (`docs/50 - 56 - Currying.md`) leaf 1. Entered once at the
-- | emitter's entry (`emit_with_map`) and restored on drop, so one file's
-- | signatures are never visible while another is being written — §1.4's
-- | boundary, enforced by the lifetime of this guard rather than by convention.
-- |
-- | # Why the table is ambient and not a parameter
-- |
-- | Both readers are *inside* expression emission — the `Expr::Call` arm and
-- | [`wraps_as_receiver`](crate::emit::wraps_as_receiver) — and `emit_expr` has
-- | no context argument at all. §11.3 forbids the two obvious alternatives for
-- | reasons that are each a measurement rather than a preference:
-- |
-- | - **Not threaded through `emit_expr`.** Twenty-four importers and
-- |   thirty-six call sites would gain a parameter that all but two of them
-- |   only pass down.
-- | - **Not a rewrite of `Program` in `parse()`.** `redox_fmt::format` consumes
-- |   the same tree (`redox-cli/src/fmt_command.rs`,
-- |   `redox-lsp/src/serve/formatting.rs`, `redox-mcp/src/format.rs`), so a
-- |   `Call`→`Lambda` rewrite there would make `redox fmt` rewrite the author's
-- |   `add 1` into a closure. Plan 61 states the rule this is a case of: *a
-- |   pass that rewrites `Program` has two consumers and will surprise one of
-- |   them.*
-- |
-- | §11.3 then names the remaining split as *"the check rides `parse()`; the
-- | rewrite lives at `emit_with_map` entry"* — and the check half went with
-- | leaf 3, which §11.1 withdrew, so nothing rides `parse()` and there is no
-- | rewrite either. What is left is the smallest thing that does the job: the
-- | decision is **local** to the Call arm, which already holds the callee and
-- | the argument list, and only the table has to travel. It travels here.
-- |
-- | A `thread_local!` rather than a `static`, so the suite's parallel tests
-- | cannot see each other's files; a guard rather than a setter, so an early
-- | return cannot leave a stale table behind.
export(crate) data ArityScope = ArityScope {
  -- | The table to put back when the guard drops.
  previous : ArityTable,
}

instance ArityScope
  -- | Install `table` for as long as the returned guard lives.
  export(crate) enter : ArityTable -> Self
  enter table =
    previous = .with_borrow_mut (\\slot -> std::mem::replace slot table) IN_SCOPE
    ArityScope { previous }

  -- | The declared arity of `name` in the file being emitted.
  export(crate) arity_of : &str -> Option<usize>
  arity_of name = .with_borrow (\\table -> .arity_of name table) IN_SCOPE

instance Drop ArityScope
  drop : &mut self -> ()
  drop self =
    restored = std::mem::take (&mut self.previous)
    -- `std::mem::replace` and a discard, where the Rust wrote `*slot =
    -- restored`: a lambda body is one *expression* and `:=` is an assignment,
    -- so the assignment has no place to sit. `replace` says the same thing and
    -- is `enter`'s own idiom one function up.
    _dropped = .with_borrow_mut (\\slot -> std::mem::replace slot restored) IN_SCOPE
    ()
''')

print('plan 81 batch 5: three files')
