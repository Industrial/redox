# -*- coding: utf-8 -*-
r"""Plan 82 batch 12: `parse/parser` — the `Parser` struct and its methods.

The last file of the last crate, and the one that makes `voltaic-parser`'s own
parser a Voltaic program.

Four things in it are worth naming up front.

**`data Parser = Parser { toks : &'a [Token], … }`** takes its lifetime the way
`voltaic-emit`'s `Sink` does: `generalise_data` reads `'a` out of the field types
and emits `struct Parser<'a>`, and `instance (Parser 'a)` is the impl head. The
`'a` stays out of every method's own generic list because
`MethodContext::for_instance` puts the target's first type variable in
`reserved`, which is exactly what that list is for.

**`..decl` has no Voltaic spelling.** `item`'s tail builds an `FnDecl` from the
one `function` returned, overriding three fields with Rust's functional record
update. Three assignments to a `mut` binding are the same thing written out,
and the `span` override reads the old `span` before replacing it — which the
update syntax also does.

**A diverging `loop` cannot be a `()` body's tail.** `recover_to_item` and
`recover_to_stmt` are `loop { … return; … }` in Rust, and plan 55 §1.6 requires
a `()`-returning body to *end* with `()`, `break`, `continue` or `()` — a
`loop` is none of them. Both become `while running`, which ends with `()` and
needs no unreachable statement after it.

**`clause`, `where_locals` and `stmt` are free functions, not methods.** The
generated `parse/parser.rs` came to **1,648 lines** against `line_cap.rs`'s
1,500 — the hand-written file was 1,482, and the emitter puts a blank line
after each `use`, which is 42 of the difference on its own.

A second `impl` in a child module was tried first and is **not** allowed here:
`one_construct_per_file` requires exactly one top-level construct per file and
an impl-only file declares none, which is doc 06 §1's rule that *"`impl` blocks
travel with their type"* doing its job. So the three largest private methods
take the shape the other 117 files in this crate already have — a free function
over `&mut Parser`, in a file named after it. They are `export(super)`, which
from a child module is the scope a private method of `parse::parser` had:
visible there and in every descendant, and nowhere else.

`where_locals` is imported by `clause` and not by `parse/parser` itself:
`clause` was its only caller and went with it.
"""
import sys

import io  # noqa: E402

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

ROOT = '/home/tom/Code/rust/voltaic/crates/voltaic-parser/sdx/'

write('parse/parser', r'''
import crate::parse::parser::clause::clause
import crate::parse::parser::stmt::stmt
import voltaic_ast::Guard

-- Three members of this impl are free functions in child modules instead,
-- for the reason this batch's docstring gives.
export(super) module clause
export(super) module stmt
export(super) module where_locals

-- | The recursive-descent parser.
-- |
-- | Private: [`crate::parse`] and [`crate::parse_source`] are the only ways in.
-- | Plan 03 (`docs/08 - 03 - Parser Core.md`) grew it from plan 01's three
-- | productions to the expression, signature and binding surface, with the
-- | expression grammar a Pratt loop over `crate::parse::precedence`.
-- |
-- | Errors recover at **two** levels: an item boundary, and a statement
-- | boundary within a block. The second is what stops one bad statement from
-- | discarding its siblings — which matters for a language whose bodies are
-- | indented blocks, and which `docs/COMPILER.md` §1 records LALRPOP never
-- | offering at all.
-- |
-- | The `'a` is **not written**: `generalise_data` reads it out of `toks`'s
-- | type and emits `struct Parser<'a>`, which is doc 18 §5.3's rule applied to
-- | a lifetime — the same way `voltaic-emit`'s `Sink` takes its own.
export(crate) data Parser = Parser {
  toks : &'a [Token],
  pos : usize,
  -- | `r#errors`, because `errors` is doc 03 §14's declaration keyword.
  r#errors : Vec<ParseError>,
  -- | Plan 63 leaf 1: every `class` member's signature head, by class and
  -- | member name, so an `instance` may omit a signature the class already
  -- | states.
  -- |
  -- | **Collect and never transform**, which is plan 61's rule about a pass
  -- | with two consumers: this is written while a `class` body is parsed and
  -- | read while an `instance` body is, and it changes no node either time.
  -- |
  -- | Populated in declaration order, which is the row's one narrowing: a
  -- | class must appear **before** the instance that elides against it.
  -- | Restating a signature still works in any order, so no program that
  -- | compiled before this row stops — see §10.
  class_heads : std::collections::BTreeMap<(String, String), SignatureHead>,
}

instance (Parser 'a)
  export(crate) new : &'a [Token] -> Self
  new toks = Parser { toks, pos = 0, r#errors = Vec::new (), class_heads = std::collections::BTreeMap::new () }
  -- | Parse the whole token stream, consuming the parser.
  export(crate) run : usize -> mut self -> Result<Program, Vec<ParseError>>
  run src_len self =
    items = .program self
    -- Doc 03 §7.6's `RDX0044` — plan 30. A whole-program pass rather than a
    -- check inside the block parser: `return_in_tail` recurses into the blocks
    -- a statement owns, so running it per block would report a nested tail's
    -- `return` twice. One entry point per body is the shape that cannot
    -- double-report.
    --
    -- Run only when parsing otherwise succeeded, so a file with a syntax error
    -- reports the syntax error rather than a second diagnostic about a tail the
    -- parser may have recovered into.
    if .is_empty self.r#errors
      probe = Program { items = .clone items, span = (0, src_len), trailing = Vec::new () }
      .extend (check_callback_tails (&probe)) self.r#errors
    if .is_empty self.r#errors
      trailing = .eof_trivia self
      return Ok (Program { items, span = (0, src_len), trailing })
    Err self.r#errors
  program : &mut self -> Vec<Item>
  program self =
    mut items = Vec::new ()
    .skip_separators self
    while !.at_eof self
      case .item self of
        Ok found -> .push found items
        Err e ->
          .push e self.r#errors
          .recover_to_item self
      .skip_separators self
    items
  -- | The items of an indented block, to the matching `Dedent`.
  -- |
  -- | Plan 14's `module` body. The same item-level recovery `program` uses —
  -- | a bad item is reported and skipped and its siblings are still parsed —
  -- | because a module body is a list of items and wants exactly that.
  export(crate) items_until_dedent : &mut self -> Result<Vec<Item>, ParseError>
  items_until_dedent self =
    mut items = Vec::new ()
    loop
      .skip_separators self
      if .check (&TokenKind::Dedent) self || .at_eof self
        break
      case .item self of
        Ok found -> .push found items
        Err e ->
          .push e self.r#errors
          .recover_to_item self
    (.expect (&TokenKind::Dedent) "expected the module body to end" self)?
    Ok items
  -- | ```text
  -- | Item := Ident ":" Spine Newline
  -- |         Ident Binder* "=" Newline Indent Stmt+ Dedent
  -- | ```
  -- |
  -- | Doc 18 §§2.2–2.3 — the signature line and the equation. Two lines where
  -- | doc 03 had one, and the trade doc 18 §10.1 item 2 flags as the thing to
  -- | watch: a one-line private helper now costs two lines. What it buys is
  -- | that the types are written once, in Haskell's order, and the equation is
  -- | a row of binders rather than a repeated annotation.
  -- |
  -- | The two names must match. Catching that here rather than at the emitter
  -- | is what makes a mistyped equation a diagnostic about the equation
  -- | instead of an undefined-function error from rustc about a line the user
  -- | did not write.
  item : &mut self -> Result<Item, ParseError>
  item self =
    leading = .peek_leading self
    start = (.peek_span self).0
    -- Doc 03 §9.6's marker comes first, before the keyword that says which
    -- declaration this is — so it is read before the dispatch rather than
    -- once per branch.
    --
    -- `marker`, where the Rust named the local `visibility`: that is the name
    -- of the function it calls, and `RDX1003` reports the shadow.
    marker = (visibility self)?
    -- Doc 18 §2.4's declarations are told apart by their first token, which
    -- is a reserved word — so one token of lookahead settles which item this
    -- is, with no backtracking and no ambiguity against a function's
    -- signature line (a function begins with a *name*).
    --
    -- One line, where the Rust wrapped the condition: a `||` opening a line
    -- is `RDX0102`.
    if .check (&TokenKind::Kw (Keyword::Data)) self || .check (&TokenKind::Kw (Keyword::Newtype)) self || .check (&TokenKind::Kw (Keyword::Union)) self
      decl = (data_decl self leading start marker)?
      (.newline_with_trailing "expected a line break after the declaration" self)?
      return Ok (Item::Data decl)
    if .check (&TokenKind::Kw (Keyword::Type)) self
      alias = (type_alias self leading start marker)?
      (.newline_with_trailing "expected a line break after the alias" self)?
      return Ok (Item::Type alias)
    if .check (&TokenKind::Kw (Keyword::Const)) self || .check (&TokenKind::Kw (Keyword::Static)) self
      decl = (const_decl self leading start marker)?
      (.newline_with_trailing "expected a line break after the binding" self)?
      return Ok (Item::Const decl)
    if .check (&TokenKind::Kw (Keyword::Import)) self
      decl = (import_decl self leading start marker)?
      (.newline_with_trailing "expected a line break after the import" self)?
      return Ok (Item::Import decl)
    if .check (&TokenKind::Kw (Keyword::Module)) self
      decl = (module_decl self leading start marker)?
      (.newline_with_trailing "expected a line break after the module" self)?
      return Ok (Item::Module decl)
    if .check (&TokenKind::Kw (Keyword::Test)) self
      decl = (test_decl self leading start)?
      (.newline_with_trailing "expected a line break after the test" self)?
      return Ok (Item::Test decl)
    -- Doc 01 §23.1's escape hatch — plan 45 leaf 1. A reserved word like the
    -- rest of this dispatch, which is what stops `rust """…"""` from parsing
    -- as a call to a function named `rust` and emitting one.
    if .check (&TokenKind::Kw (Keyword::Rust)) self
      block = (inline_rust self leading start)?
      (.newline_with_trailing "expected a line break after the inline Rust" self)?
      return Ok (Item::InlineRust block)
    -- Doc 01 §26's FFI requirement — plan 45 leaf 3.
    if .check (&TokenKind::Kw (Keyword::Extern)) self
      block = (extern_block self leading start)?
      return Ok (Item::Extern block)
    if .check (&TokenKind::Kw (Keyword::Errors)) self
      decl = (errors_decl self leading start marker)?
      (.newline_with_trailing "expected a line break after the declaration" self)?
      return Ok (Item::Errors decl)
    if .check (&TokenKind::Kw (Keyword::Class)) self
      decl = (class_decl self leading start marker)?
      (.newline_with_trailing "expected a line break after the class" self)?
      return Ok (Item::Class decl)
    -- An `impl` has no visibility in Rust and a test is never public, so
    -- neither takes the marker — and a marker before one is rejected by the
    -- `=` that follows, which is the right error for writing it.
    if .check (&TokenKind::Kw (Keyword::Instance)) self
      decl = (instance_decl self leading start)?
      (.newline_with_trailing "expected a line break after the instance" self)?
      return Ok (Item::Instance decl)
    -- Three assignments, where the Rust wrote `..decl`: functional record
    -- update has no Voltaic spelling, and overriding three fields of a `mut`
    -- binding is the same thing written out. `span` reads the old span before
    -- replacing it, exactly as the update syntax does.
    mut decl = (.function (&[]) self)?
    decl.leading := leading
    decl.visibility := marker
    decl.span := (start, decl.span.1)
    Ok (Item::Fn decl)
  -- | A signature line and its equations — doc 18 §§2.2–2.3.
  -- |
  -- | Lifted out of [`Parser::item`] when doc 18 §2.5's class and instance
  -- | bodies arrived, because a method **is** a function: the same signature
  -- | line, the same equations, the same guard ladders. Sharing the code is
  -- | what gives a default method several clauses and a guard ladder without
  -- | anything new being written for it.
  -- |
  -- | `reserved` names type variables belonging to the *enclosing*
  -- | declaration rather than to this function — a class's variable, which
  -- | must not become a generic parameter of each of its methods. A top-level
  -- | function passes nothing.
  -- |
  -- | **Zero equations is legal here**, and that is the one difference from a
  -- | top-level function. `area : &t -> f64` with no body is a *required*
  -- | method, which Rust writes `fn area(&self) -> f64;` — so the caller, not
  -- | this, decides whether a bodiless signature is an error.
  export(crate) function : &[String] -> &mut self -> Result<FnDecl, ParseError>
  function reserved self =
    leading = .peek_leading self
    start = (.peek_span self).0
    name = (.ident "expected a name to begin a declaration" self)?
    head = (.signature_head (&name) self)?
    .equations name leading start (&head) reserved false self
  -- | A function declared inside a `where` group — plan 61 leaf 0.
  -- |
  -- | [`Self::function`] with plan 63's `ends_on_a_new_name` **set**, which is
  -- | the only difference and is load-bearing. At module scope and at the top
  -- | of a `class` body a line beginning with a different name is a new
  -- | declaration, and the caller reads it; inside a `where` group it is a
  -- | **binding**, so without the flag `equations` consumes it as another
  -- | equation of the function above and reports `RDX0103` *"this equation
  -- | defines `trebled`, but the signature above it is for `half`"*.
  -- |
  -- | Plan 63 leaf 0 added the flag for the same shape one construct over —
  -- | two consecutive elided class members colliding — and its four lines are
  -- | the whole fix here too.
  export(crate) where_function : &mut self -> Result<FnDecl, ParseError>
  where_function self =
    leading = .peek_leading self
    start = (.peek_span self).0
    name = (.ident "expected a name to begin a declaration" self)?
    head = (.signature_head (&name) self)?
    .equations name leading start (&head) (&[]) true self
  -- | A signature line, from the `:` to the line break.
  -- |
  -- | Plan 63 (`docs/57 - 63 - Instance Inference.md`) leaf 0. Extracted from
  -- | [`Self::function`] so that an `instance` member which **omits** its
  -- | signature can skip this and borrow a class's head instead — without a
  -- | second copy of the markers, the constraint clause and the spine, each of
  -- | which has an ordering rule that took a plan to get right.
  export(crate) signature_head : &Ident -> &mut self -> Result<SignatureHead, ParseError>
  signature_head name self =
    -- Doc 01 §4.3's `PI = 3.14159`, which doc 03 §13.4 corrected out of
    -- existence. The generic "expected `:`" is true and unhelpful: the
    -- author followed a published specification, so the message names the
    -- correction instead of the token.
    --
    -- **The value has to be on the same line.** `main =` followed by a line
    -- break is a *function* whose signature line is missing, and that wants
    -- the `:` message — the first version of this check fired on it too and
    -- `reports_a_missing_signature_marker` caught it.
    binds_on_this_line = .check (&TokenKind::Eq) self && .is_some_and (\t -> t.kind != TokenKind::Newline) (.peek_at 1 self)
    if binds_on_this_line
      message = format! \-> "a module-scope binding needs `const` or `static` and a type: `const #{name.text} : T = …`. Rust forbids `let` at module scope, and doc 03 §13.4 corrects doc 01 §4.3 accordingly"
      return Err (ParseError::r#with Code::ModuleScopeBinding message name.span)
    (.expect (&TokenKind::Colon) "expected `:` after the name, to begin the signature" self)?
    -- Doc 18 §2.2's `=>` clause. Read speculatively, because
    -- `f : Ord t => &[t] -> &t` and `f : &[t] -> &t` begin with the same
    -- token — see `constraints`.
    bounds = (constraints self)?
    -- Doc 18 §2.2's `const fn` marker, consumed **before** the spine: a type
    -- is verbatim text, so leaving `const` in one would make the first
    -- parameter's type `const u32`, which is not a Rust type.
    is_const = .eat_keyword Keyword::Const self
    -- Doc 03 §14.3's `async fn` — plan 39. Read here for `const`'s reason:
    -- left in a type it would produce `async u64`, and read in either order
    -- because Rust accepts neither pairing anyway.
    is_async = .eat_keyword Keyword::Async self
    -- Rust's `unsafe fn` — plan 45 leaf 2. Read beside the other two markers
    -- for their reason: a type is verbatim text, and `unsafe` left in one
    -- would make the first parameter's type `unsafe u32`.
    is_unsafe = .eat_keyword Keyword::Unsafe self
    spine = (parse_spine self)?
    (.newline_with_trailing "expected a line break after the signature" self)?
    Ok (SignatureHead { types = spine, bounds, is_const, is_async, is_unsafe })
  -- | Record a `class` member's head, so an `instance` may elide it.
  -- |
  -- | Plan 63 leaf 1, and the narrowest resolver the row could have: one name,
  -- | one kind of declaration, one file, **top-level items only** — §11.3's
  -- | decision. A class the lookup does not find degrades to today's
  -- | behaviour, which is a program that still compiles, so the narrow version
  -- | cannot regress anything.
  export(crate) record_head : &str -> &str -> &SignatureHead -> &mut self -> ()
  record_head class_name member head self =
    key = (.to_string class_name, .to_string member)
    .insert key (.clone head) self.class_heads
    ()
  -- | The head a `class` states for `member`, if this file has declared it.
  export(crate) borrowed_head : &str -> &str -> &self -> Option<SignatureHead>
  borrowed_head class_name member self =
    key = (.to_string class_name, .to_string member)
    .cloned (.get (&key) self.class_heads)
  -- | A declaration's equations, under a signature head that is **already
  -- | read** — from this declaration's own signature line, or borrowed from a
  -- | class.
  -- |
  -- | Plan 63 (`docs/57 - 63 - Instance Inference.md`) leaf 0. Extracted from
  -- | [`Self::function`] rather than copied, because an `instance` member that
  -- | omits its signature needs exactly this loop and nothing else — and two
  -- | copies of the clause grammar is how doc 18 §2.3's multi-equation rule
  -- | would start disagreeing with itself.
  export(crate) equations : Ident -> Vec<Trivia> -> usize -> &SignatureHead -> &[String] -> bool -> &mut self -> Result<FnDecl, ParseError>
  equations name leading start head reserved ends_on_a_new_name self =
    spine = &head.types
    bounds = &head.bounds
    -- The equations. Each repeats the name — doc 18 §2.3.
    mut clauses = Vec::new ()
    mut sig = None
    loop
      -- An equation begins with a name that is *not* followed by `:` — a
      -- name that is starts the next declaration's signature line. That one
      -- test tells three things apart: another clause of this function, a
      -- new declaration, and (doc 18 §2.5) a required method with no
      -- equation at all.
      is_equation = case &(.peek self).kind of
        TokenKind::Ident _ -> !.is_some_and (\t -> t.kind == TokenKind::Colon) (.peek_at 1 self)
        _ -> false
      if !is_equation
        break
      -- Plan 63 leaf 0. Inside a `class` or `instance` body a differing name
      -- starts the **next member**, where at top level it is a mistyped
      -- equation. One test, two readings, and the body is the only caller
      -- that can tell them apart — so the caller says which it is.
      --
      -- Without this, two consecutive members that both elide their
      -- signatures collide: `area self = …` then `describe self = …` reads
      -- the second as a continuation of the first and the guard below fires
      -- `RDX0103`. `examples/class.sdx` is the file that proved it.
      same_name = case &(.peek self).kind of
        TokenKind::Ident text | text == &name.text -> true
        _ -> false
      if ends_on_a_new_name && !same_name
        break
      again = (.ident "expected the equation to repeat the name" self)?
      -- The two names must match, and catching it here is what makes a
      -- mistyped equation a diagnostic *about the equation* rather than an
      -- undefined-function error from rustc about a line the user did not
      -- write. An equation whose name differs is reported rather than left
      -- to be misread as a new declaration, which is what it looks like to
      -- one token of lookahead.
      if again.text != name.text
        message = format! \-> "this equation defines `#{again.text}`, but the signature above it is for `#{name.text}`"
        return Err (ParseError::r#with Code::ExpectedName message again.span)
      row = (patterns self)?
      -- Every clause is checked against the same spine, so a row of the wrong
      -- length is reported per clause rather than once for the function.
      built = (signature_from spine (&row) (again.span.0, (.prev_span self).1))?
      -- Plan 55 §1.6's unit rule needs the declared return type, which only
      -- this knows, and the body's *shape*, which only `clause` knows — so
      -- the bit travels down rather than the block coming back up.
      returns_unit = .returns_unit built
      if .is_none sig
        sig := Some built
      .push (clause row (&name) returns_unit self)? clauses
      .skip_separators self
    -- With no equation there is no pattern row, so the signature comes from
    -- the spine alone and the parameters take generated names. Not
    -- `signature_from` with an empty row: that reports an arity error, which
    -- is the right answer for `area = …` under `area : &t -> f64` and the
    -- wrong one for a required method with no equation at all.
    --
    -- `settled`, where the Rust shadowed `sig` with its own unwrap.
    mut settled = .unwrap_or_else (\-> signature_only spine (name.span.0, (.prev_span self).1)) sig
    -- The generic list is synthesised from the types, not written — doc 18
    -- §5.3. Done here rather than in the two signature builders because it
    -- needs the constraint clause, which only this knows about.
    generalise (&mut settled) bounds reserved
    settled.is_const := head.is_const
    settled.is_async := head.is_async
    settled.is_unsafe := head.is_unsafe
    end = (.prev_span self).1
    trailing = .and_then (\c -> .clone c.trailing) (.last clauses)
    -- A method's marker is read by `class_member`; a top-level function's
    -- by `item`, which replaces this.
    Ok (FnDecl { name, sig = settled, clauses, visibility = Visibility::Private, span = (start, end), leading, trailing })
  -- | The statements of an indented block, from just inside the `Indent` to
  -- | the matching `Dedent`.
  -- |
  -- | Lifted out of [`Parser::clause`] when doc 03 §15's `test` block arrived,
  -- | because a test's body **is** a clause body: same statements, same
  -- | recovery, same emptiness rule. Sharing it is what gives a test the draw
  -- | arrows and `case` expressions that need layout of their own, with
  -- | nothing written for them.
  -- |
  -- | Statement-level recovery lives here: a bad statement is reported and
  -- | skipped, and its siblings are still parsed.
  statements : &mut self -> Result<Block, ParseError>
  statements self =
    start = (.peek_span self).0
    mut stmts = Vec::new ()
    loop
      .skip_separators self
      if .check (&TokenKind::Dedent) self || .at_eof self
        break
      case stmt self of
        Ok found -> .push found stmts
        Err e ->
          .push e self.r#errors
          .recover_to_stmt self
    end = (.prev_span self).1
    (.expect (&TokenKind::Dedent) "expected the body to end" self)?
    if .is_empty stmts && .is_empty self.r#errors
      return Err (ParseError::r#with Code::EmptyBody "a declaration body must contain at least one statement" (start, start))
    Ok (Block { stmts, span = (start, end) })
  -- | An `Indent`, its statements, and the matching `Dedent`.
  -- |
  -- | What a caller wants when the block is the whole construct — a `test`
  -- | body — rather than one of several shapes a clause can take.
  export(crate) indented_block : Code -> &str -> &mut self -> Result<Block, ParseError>
  indented_block code message self =
    (.expect_with code (&TokenKind::Indent) message self)?
    .statements self
  -- | An indented block of statements, for a construct whose body is one.
  -- |
  -- | `indented_block` with a line break in front of it, which every
  -- | control-flow body needs and no other caller does: an `if`'s condition
  -- | ends its line, and the block opens on the next.
  export(crate) statement_block : &str -> &mut self -> Result<Block, ParseError>
  statement_block message self =
    (.newline_with_trailing "expected a line break before the body" self)?
    .indented_block Code::EmptyBody message self
  -- | Whether the statement ends here — a line break, a dedent, or the end of
  -- | the file.
  -- |
  -- | What tells `break` from `break v`, and the same test `return` already
  -- | made inline. Named once now that two statements ask it.
  export(crate) at_statement_end : &self -> bool
  at_statement_end self = .check (&TokenKind::Newline) self || .check (&TokenKind::Dedent) self || .at_eof self
  -- | `if c` with an indented block, and its `else` — doc 01 §8, plan 20.
  -- |
  -- | `else if` is read by recursion: the `else` branch is a block holding one
  -- | `If` statement, which is how Rust reads it and what lets the emitter
  -- | write `} else if … {` without a third node.
  -- |
  -- | **The only `if` production, since plan 55 leaf 7 retired `then`.**
  -- | `prefix` reached this or `if_expression` depending on a depth-zero scan
  -- | for the keyword (`reaches_a_then`, gone with it); with one spelling left
  -- | the expression position is the same parse as the statement position, and
  -- | the choice — along with `Expr::If`'s `is_inline` bit — has nothing to
  -- | record.
  export(crate) if_statement : &mut self -> Result<Expr, ParseError>
  if_statement self =
    open = (.peek_span self).0
    (.expect (&TokenKind::Kw (Keyword::If)) "expected `if`" self)?
    -- `cond` and `fallback`, where the Rust named them `condition` and
    -- `otherwise`: the first is this file's imported function and the second
    -- is doc 03 §7.5's keyword. Both fields keep their own names.
    cond = (condition self)?
    then_block = (.statement_block "expected an indented `if` body" self)?
    .skip_separators self
    if !.check (&TokenKind::Kw (Keyword::Else)) self
      return Ok (Expr::If { condition = cond, then_block, r#otherwise = None, span = (open, (.prev_span self).1) })
    .advance self
    -- `else if` — one statement in a block of its own, so the chain is a
    -- chain and the emitter needs no special case.
    if .check (&TokenKind::Kw (Keyword::If)) self
      start = (.peek_span self).0
      nested = (.if_statement self)?
      end = (.prev_span self).1
      only = Stmt { expr = nested, span = (start, end), leading = Vec::new (), trailing = None }
      chained = Block { stmts = vec! \-> only, span = (start, end) }
      return Ok (Expr::If { condition = cond, then_block, r#otherwise = Some chained, span = (open, end) })
    fallback = (.statement_block "expected an indented `else` body" self)?
    Ok (Expr::If { condition = cond, then_block, r#otherwise = Some fallback, span = (open, (.prev_span self).1) })
  -- | One expression as a body, for a clause or guard written on one line.
  -- |
  -- | A block of one tail statement, so the emitter has one body shape to
  -- | handle rather than two — the inline form is a spelling, not a second
  -- | kind of body.
  inline_body : &mut self -> Result<Block, ParseError>
  inline_body self =
    start = (.peek_span self).0
    value = (parse_expr self 0)?
    end = (.prev_span self).1
    only = Stmt { expr = value, span = (start, end), leading = Vec::new (), trailing = None }
    Ok (Block { stmts = vec! \-> only, span = (start, end) })
  -- | Whether a guard list follows: a line break, an indent, then `|`.
  -- |
  -- | Three tokens of lookahead rather than one, because a line break after
  -- | the pattern row is also what a *forgotten* `=` looks like. Deciding on
  -- | the break alone made `add a b` with no `=` report "expected at least one
  -- | `|` guard" and then cascade into two more errors as the body was read as
  -- | fresh items — a worse diagnostic than the one it replaced, for the
  -- | likelier mistake. Doc 05 §4's rule is that the first error names the
  -- | actual defect.
  opens_a_guard_list : &self -> bool
  opens_a_guard_list self = .check (&TokenKind::Newline) self && .is_some_and (\t -> t.kind == TokenKind::Indent) (.peek_at 1 self) && .is_some_and (\t -> t.kind == TokenKind::Or) (.peek_at 2 self)
  -- | A guard ladder: `| cond = body` rows, in order.
  -- |
  -- | `otherwise` is Haskell's name for the final catch-all and is recognised
  -- | here as a condition of `None`, so the emitter can write a bare `else`.
  -- | Lowering it to `if true` instead would leave rustc unable to see that the
  -- | ladder is total, and every guarded function would need a return the
  -- | author did not write.
  guards : &mut self -> Result<Vec<Guard>, ParseError>
  guards self =
    mut rows = Vec::new ()
    loop
      .skip_separators self
      if !.check (&TokenKind::Or) self
        break
      start = (.peek_span self).0
      .advance self
      -- `otherwise` is a reserved word (doc 18 §7), not an identifier the
      -- parser compares against: `| otherwise = …` must not be readable as a
      -- comparison against a variable someone happened to name `otherwise`.
      cond = if .check (&TokenKind::Kw (Keyword::Otherwise)) self
        .advance self
        None
      else
        Some (parse_expr self 0)?
      (.expect (&TokenKind::Eq) "expected `=` after a guard's condition" self)?
      body = (.inline_body self)?
      (.newline_with_trailing "expected a line break after a guard's body" self)?
      .push (Guard { condition = cond, body, span = (start, (.prev_span self).1) }) rows
    debug_assert! \-> !.is_empty rows \-> "`opens_a_guard_list` guarantees a first `|`"
    Ok rows
  -- | Doc 30 §3.3's callback bind — `with conn = pool.with_connection`.
  -- |
  -- | Doc 03 §7.6's lowering, unchanged: *"rewrites the remainder of the
  -- | enclosing block into a closure passed as the final argument"*. Only the
  -- | spelling has moved, from §7.6's `use x <- f` via doc 18 §3.1's `x <- f`
  -- | to this — because `<-` now means `await` and nothing else.
  -- |
  -- | Recognised by the keyword, so no lookahead at all. Doc 18 §3.1's version
  -- | needed one token of it and a hard-won lexer fix: `x <- f` once lexed as
  -- | `<` then `-`, parsed as `x < -f`, and emitted valid Rust comparing `x`
  -- | against negated `f` with no diagnostic. A keyword cannot do that.
  try_draw : &mut self -> Result<Option<Expr>, ParseError>
  try_draw self =
    if !.check (&TokenKind::Kw (Keyword::With)) self
      return Ok None
    start = (.peek_span self).0
    .advance self
    name = (.ident "expected a name after `with`" self)?
    (.expect (&TokenKind::Eq) "expected `=` after a `with` binder — doc 30 §3.3" self)?
    value = Box::new (parse_expr self 0)?
    Ok (Some (Expr::Draw { name, value, span = (start, (.prev_span self).1) }))
  -- | A binding, if the cursor is on one.
  -- |
  -- | Recognised by lookahead rather than by backtracking: `mut` opens one
  -- | outright, and otherwise an `Ident` followed by `=` or `:` is a binding
  -- | while anything else is an expression statement.
  try_binding : &mut self -> Result<Option<Binding>, ParseError>
  try_binding self =
    is_mut = .check (&TokenKind::Kw (Keyword::Mut)) self
    if !is_mut && !.looks_like_a_binding self
      return Ok None
    start = (.peek_span self).0
    if is_mut
      .advance self
    -- Doc 03 §6.5's destructuring — plan 33. A plain name is a
    -- `Pattern::Binding` and takes the path it always took.
    bound = (pattern_alt self)?
    ty = if .check (&TokenKind::Colon) self
      .advance self
      Some (.type_expr "expected a type after `:`" self)?
    else
      None
    (.expect (&TokenKind::Eq) "expected `=` in a binding" self)?
    value = (parse_expr self 0)?
    Ok (Some (Binding { pattern = bound, ty, value, is_mut, span = (start, (.prev_span self).1) }))
  -- | Whether a binding starts here — plan 33.
  -- |
  -- | A plain binding is an identifier then `=` or `:`, which is one token of
  -- | lookahead. A **destructuring** binding opens with `(`, `[` or an
  -- | uppercase name followed by `{`, and its `=` is an unknown distance away —
  -- | so those three openers are recognised by their first token and the `=` is
  -- | found by scanning to it at depth zero.
  -- |
  -- | Scanning rather than backtracking, for the reason plan 22's condition
  -- | parser gives: a lookahead that reads token kinds and advances nothing
  -- | cannot leave the parser somewhere unexpected when it answers no.
  looks_like_a_binding : &self -> bool
  looks_like_a_binding self = case &(.peek self).kind of
    TokenKind::Ident _ ->
      -- An `if` over a `case`, where the Rust wrote one `matches!` over an
      -- `Option<&TokenKind>`: a `case` needs a line of its own.
      opener = if .peek_at 1 self is Some next
        case next.kind of
          TokenKind::Eq or TokenKind::Colon or TokenKind::LBrace -> true
          _ -> false
      else
        false
      opener && .reaches_a_binding_eq self
    TokenKind::LParen or TokenKind::LBracket -> .reaches_a_binding_eq self
    _ -> false
  -- | Whether a top-level `=` appears before this statement ends.
  -- |
  -- | What tells `(a, b) = p` from the expression `(a, b)`, and `Point { x } = p`
  -- | from the record *literal* `Point { x = 1 }` — the literal's `=` is inside
  -- | its braces, so it is not at depth zero.
  export(crate) reaches_a_binding_eq : &self -> bool
  reaches_a_binding_eq self =
    mut depth : i32 = 0
    mut i = 0
    while .peek_at i self is Some tok
      case &tok.kind of
        TokenKind::Eq | depth == 0 ->
          return true
        TokenKind::LParen or TokenKind::LBracket or TokenKind::LBrace -> depth += 1
        TokenKind::RParen or TokenKind::RBracket or TokenKind::RBrace ->
          depth -= 1
          if depth < 0
            return false
        TokenKind::Newline or TokenKind::Indent or TokenKind::Dedent or TokenKind::Eof ->
          return false
        _ -> ()
      i += 1
    false
  -- | Capture a type as verbatim source text (plan 03 §1.1), reading to its
  -- | terminator.
  export(crate) type_expr : &str -> &mut self -> Result<TypeExpr, ParseError>
  type_expr message self = capture_type self message TypeStop::Greedy

  -- ---- token helpers, shared with the grammar functions ----
  export(crate) peek : &self -> &'a Token
  peek self =
    -- `lex` always terminates the stream with `Eof`, so the last token is a
    -- safe clamp rather than an out-of-bounds risk.
    .unwrap_or_else (\-> .expect "the token stream always contains at least Eof" (.last self.toks)) (.get self.pos self.toks)
  -- | The token `n` positions ahead — the bounded lookahead `try_binding`
  -- | needs, and which plan 19's call grammar will need too.
  export(crate) peek_at : usize -> &self -> Option<&'a Token>
  peek_at n self = .get (self.pos + n) self.toks
  export(crate) peek_span : &self -> Span
  peek_span self = (.peek self).span
  peek_leading : &self -> Vec<Trivia>
  peek_leading self = .clone (.peek self).leading
  export(crate) prev_span : &self -> Span
  prev_span self =
    if self.pos == 0
      return (0, 0)
    self.toks[self.pos - 1].span
  export(crate) at_eof : &self -> bool
  at_eof self = case (.peek self).kind of
    TokenKind::Eof -> true
    _ -> false
  export(crate) check : &TokenKind -> &self -> bool
  check kind self = &(.peek self).kind == kind
  export(crate) advance : &mut self -> ()
  advance self =
    if !.at_eof self
      self.pos += 1
    ()
  export(crate) expect : &TokenKind -> &str -> &mut self -> Result<(), ParseError>
  expect kind message self = .expect_with Code::ExpectedToken kind message self
  -- | [`Parser::expect`] with a code other than [`Code::ExpectedToken`].
  -- |
  -- | A missing token is usually nothing more specific than "the grammar
  -- | wanted something else here", but a few are worth their own code because
  -- | a reader recognises them: a declaration with no body is `RDX0105`, not a
  -- | generic missing `Indent`.
  export(crate) expect_with : Code -> &TokenKind -> &str -> &mut self -> Result<(), ParseError>
  expect_with code kind message self =
    if .check kind self
      .advance self
      return Ok ()
    Err (ParseError::r#with code message (.peek_span self))
  -- | Consume `kw` if it is there, reporting whether it was.
  export(crate) eat_keyword : Keyword -> &mut self -> bool
  eat_keyword kw self =
    if .check (&TokenKind::Kw kw) self
      .advance self
      return true
    false
  -- | The current position, for a speculative read that may have to rewind.
  -- |
  -- | Doc 18 §2.5's constraint clause needs it: `class Shape t` and
  -- | `class Debug t => Shape t` begin identically, so the constraints are
  -- | read and then abandoned if no `=>` arrives. Exposed as an opaque mark
  -- | rather than as the index it is, so a caller cannot do arithmetic on it.
  export(crate) position : &self -> usize
  position self = self.pos
  -- | Return to a position taken by [`Parser::position`].
  export(crate) rewind : usize -> &mut self -> ()
  rewind mark self =
    self.pos := mark
    ()
  -- | The identifier here, if there is one, without erroring if there is not.
  export(crate) try_ident : &mut self -> Option<Ident>
  try_ident self =
    if &(.peek self).kind is TokenKind::Ident text
      found = Ident { text = .clone text, span = .peek_span self }
      .advance self
      return Some found
    None
  export(crate) ident : &str -> &mut self -> Result<Ident, ParseError>
  ident message self =
    if &(.peek self).kind is TokenKind::Ident text
      found = Ident { text = .clone text, span = .peek_span self }
      .advance self
      return Ok found
    Err (ParseError::r#with Code::ExpectedName message (.peek_span self))
  -- | Consume a `Newline`, returning any comment that trailed its line.
  newline_with_trailing : &str -> &mut self -> Result<Option<Trivia>, ParseError>
  newline_with_trailing message self =
    if .check (&TokenKind::Newline) self
      trailing = .clone (.peek self).trailing
      .advance self
      return Ok trailing
    -- A file ending without a trailing newline, or a block whose last
    -- statement is immediately followed by a dedent, is well-formed — and
    -- so is a statement whose expression *was* a block.
    if .at_eof self || .check (&TokenKind::Dedent) self || .just_dedented self
      return Ok None
    Err (ParseError::expected message (.peek_span self))
  -- | Whether the token just consumed closed an indented block.
  -- |
  -- | Doc 18 §2.3's `case … of` is the first expression with layout of its
  -- | own: its arms are an indented block, and the `Dedent` that ends them has
  -- | already ended the line. A caller that then demanded a `Newline` would
  -- | reject `label age = case age of …` — which is exactly how this was
  -- | found, by writing §2.3's example into an example file.
  -- |
  -- | Expressed as "the previous token was a `Dedent`" rather than "the body
  -- | was a `case`", because the next construct with its own layout should not
  -- | have to be added here too.
  export(crate) just_dedented : &self -> bool
  just_dedented self =
    self.pos
      |> .checked_sub 1
      |> .and_then (\i -> .get i self.toks)
      |> .is_some_and (\t -> t.kind == TokenKind::Dedent)
  skip_separators : &mut self -> ()
  skip_separators self =
    while .check (&TokenKind::Newline) self
      .advance self
    ()
  -- | Skip every layout token — `Newline`, `Indent` *and* `Dedent`.
  -- |
  -- | For the inside of a bracket, and only there. Plan 65 leaf 1 gave `(`,
  -- | `[` and `{` a layout context, so a construct spanning lines inside one
  -- | now carries layout where it previously carried none: `[ 1` with `, 2` on
  -- | the next line arrives as `[ 1 Newline Indent , 2 … Dedent ]`. The levels
  -- | a bracket's interior opens are closed by its closer, so a list's item
  -- | boundaries have nothing to do with them.
  -- |
  -- | Deliberately **not** [`Self::skip_separators`], which every block
  -- | construct calls: `statements` breaks on a `Dedent` and `items_until_dedent`
  -- | is named for one. Widening that function would make every indented block
  -- | in the language unterminated, which is the opposite error to the one this
  -- | fixes.
  export(crate) skip_layout : &mut self -> ()
  skip_layout self =
    while .check (&TokenKind::Newline) self || .check (&TokenKind::Indent) self || .check (&TokenKind::Dedent) self
      .advance self
    ()
  -- | Trivia left over at `Eof` — a comment at the end of the file.
  eof_trivia : &self -> Vec<Trivia>
  eof_trivia self =
    guard .last self.toks is Some last else
      return Vec::new ()
    ends = case last.kind of
      TokenKind::Eof -> true
      _ -> false
    if !ends
      return Vec::new ()
    .clone last.leading
  -- | Skip to the next item boundary: a top-level name with the indent stack
  -- | unwound.
  -- |
  -- | `while running`, where the Rust wrote `loop { … return; … }`: plan 55
  -- | §1.6 requires a `()`-returning body to end with `()`, `return`, `break`
  -- | or `continue`, and a diverging `loop` is none of the four.
  recover_to_item : &mut self -> ()
  recover_to_item self =
    mut depth : usize = 0
    mut running = true
    while running
      case &(.peek self).kind of
        TokenKind::Eof -> running := false
        TokenKind::Indent ->
          depth += 1
          .advance self
        TokenKind::Dedent ->
          depth := .saturating_sub 1 depth
          .advance self
        TokenKind::Ident _ | depth == 0 -> running := false
        _ -> .advance self
    ()
  -- | Skip to the end of the current statement, so its siblings still parse.
  recover_to_stmt : &mut self -> ()
  recover_to_stmt self =
    mut depth : usize = 0
    mut running = true
    while running
      case &(.peek self).kind of
        TokenKind::Eof -> running := false
        TokenKind::Newline | depth == 0 ->
          .advance self
          running := false
        TokenKind::Dedent | depth == 0 -> running := false
        TokenKind::Indent ->
          depth += 1
          .advance self
        TokenKind::Dedent ->
          depth := .saturating_sub 1 depth
          .advance self
        _ -> .advance self
    ()
''')

# **These three are created, not translated.** `port.sh` rebuilds `sdx/` from
# the pinned `src/`, which has no file for any of them — so each is written
# here, the way plan 80 added `check/scratch_dir/next.sdx`.
NEW = {}
NEW['parse/parser/clause'] = r'''import crate::ParseError
import crate::parse::parser::Parser
import crate::parse::parser::unit_tail::unit_tail
import crate::parse::parser::where_locals::where_locals
import voltaic_ast::{Clause, ClauseBody, Ident, Pattern}
import voltaic_diag::Code
import voltaic_lexer::TokenKind

-- | One clause's right-hand side, after its pattern row.
-- |
-- | Either `= body` or doc 18 §2.3's guard ladder. A guard list arrives on
-- | the *next* line, indented, so the newline and `Indent` are consumed
-- | before the `|` can be seen — which is why the two forms are told apart
-- | after the layout tokens rather than before them.
-- |
-- | `name` and `returns_unit` are the function's, carried in for plan 55
-- | §1.6's unit rule: the rule is decided by the declared return type and
-- | applies to an **indented** body only, so the one place that can ask it is
-- | the branch below that reads one.
-- |
-- | `row`, where the Rust named the parameter `patterns`: that is the name
-- | of the function this file imports, and `RDX1003` reports the shadow.
export(super) clause : Vec<Pattern> -> &Ident -> bool -> &mut Parser -> Result<Clause, ParseError>
clause row name returns_unit p =
  start = .map_or (.peek_span p).0 (\first -> (.span first).0) (.first row)
  if .opens_a_guard_list p
    .advance p
    .advance p
    rungs = (.guards p)?
    (.expect (&TokenKind::Dedent) "expected the guard list to end" p)?
    wheres = (where_locals (&row) p)?
    return Ok (Clause { patterns = row, body = ClauseBody::Guarded rungs, wheres, span = (start, (.prev_span p).1), trailing = None })
  (.expect (&TokenKind::Eq) "expected `=` after the equation's parameters, or an indented `|` guard list" p)?
  -- Doc 18 §2.3's own examples put short bodies on the `=` line — `fib 0 =
  -- 0` — and reserve the indented block for bodies that need statements.
  -- Both are the same construct, so both are read here rather than in two
  -- places that could drift apart.
  if !.check (&TokenKind::Newline) p
    body = (.inline_body p)?
    trailing = (.newline_with_trailing "expected a line break after the body" p)?
    wheres = (where_locals (&row) p)?
    return Ok (Clause { patterns = row, body = ClauseBody::Plain body, wheres, span = (start, (.prev_span p).1), trailing })
  trailing = (.newline_with_trailing "expected a line break after `=`" p)?
  (.expect_with Code::EmptyBody (&TokenKind::Indent) "expected an indented body" p)?
  reported = .len p.r#errors
  block = (.statements p)?
  -- Plan 55 §1.6's unit rule, and only of a block that parsed cleanly. The
  -- error count is how that is told: `statements` recovers a bad element
  -- into `p.errors` and carries on, so a body with a fault already
  -- reported still has a last element — and doc 05 §4's rule is that the
  -- first error names the actual defect, not that the second one names the
  -- tail of a body nobody has fixed yet.
  if returns_unit && .len p.r#errors == reported
    (unit_tail name (&block))?
  wheres = (where_locals (&row) p)?
  Ok (Clause { patterns = row, body = ClauseBody::Plain (.clone block), wheres, span = (start, (.prev_span p).1), trailing })
'''
NEW['parse/parser/where_locals'] = r'''import crate::ParseError
import crate::parse::parser::Parser
import crate::parse::parser::stmt::stmt
import crate::parse::parser::where_entry_is_a_function::where_entry_is_a_function
import crate::parse::parser::where_group::where_group
import voltaic_ast::{Expr, Item, Pattern, Stmt}
import voltaic_diag::Code
import voltaic_lexer::{Keyword, TokenKind}

-- | Doc 18 §2.2's `where` locals, if the clause has any.
-- |
-- | ```text
-- | largest xs = best
-- |   where
-- |     best = xs |> iter |> max |> unwrap
-- | ```
-- |
-- | An indented `where`, then its own indented block of entries. Two levels
-- | because that is how Haskell writes it and how the doc's example is
-- | written.
-- |
-- | **An entry is a binding, a function declaration, or a statement** — plan
-- | 61 leaf 0. The first two are what doc 03 §8.3 always specified; the third
-- | is what this function accepted by reusing `stmt`, and plan 61 §11.2
-- | found it reaches further than §6 claimed: *"the inner block is an
-- | ordinary statement block, so a local may be anything a statement may
-- | be"* is the sentence this comment used to end with, and it let a `print`
-- | into a group the row goes on to **sort**.
-- |
-- | A function entry is `Parser::function` reused wholesale, so a local
-- | declaration and a module-scope one are the same parse and cannot drift:
-- | multi-equation bodies, guard ladders, `where`-within-`where` and
-- | generics all arrive without being written again. It is carried as
-- | `Expr::Item`, which plan 55 leaf 5 already built and `emit_stmt` already
-- | routes to `emit_item` — so leaf 3's *"nothing in the emitter emits an
-- | item inside a function body today"* was true when it was written and is
-- | not now.
-- |
-- | Returns an empty vector when there is no clause, so a caller can ask
-- | unconditionally — there are three clause shapes and each has to.
export(super) where_locals : &[Pattern] -> &mut Parser -> Result<Vec<Stmt>, ParseError>
where_locals params p =
  opens = .check (&TokenKind::Indent) p && .is_some_and (\t -> t.kind == TokenKind::Kw (Keyword::Where)) (.peek_at 1 p)
  if !opens
    return Ok (Vec::new ())
  .advance p
  .advance p
  (.newline_with_trailing "expected a line break after `where`" p)?
  (.expect_with Code::EmptyBody (&TokenKind::Indent) "expected indented `where` bindings" p)?
  mut locals = Vec::new ()
  loop
    .skip_separators p
    if .check (&TokenKind::Dedent) p || .at_eof p
      break
    -- Plan 61 leaf 0. Asked before `stmt` because a function entry's
    -- first token is a *name*, which is the one thing `stmt` cannot tell
    -- apart on its own: `looks_like_a_binding` wants `=`, `:` or `{` at the
    -- second token, so `go 0 = 0` parsed as a call and the `=` was
    -- unexpected.
    if where_entry_is_a_function p
      start = (.peek_span p).0
      decl = (.where_function p)?
      end = (.prev_span p).1
      -- `leading` is empty for `Parser::stmt`'s reason, in the same words:
      -- `item` captured this trivia a moment ago and the declaration owns
      -- it, so a copy here printed every comment twice.
      entry = Expr::Item { item = Box::new (Item::Fn decl), span = (start, end) }
      .push (Stmt { expr = entry, span = (start, end), leading = Vec::new (), trailing = None }) locals
      continue
    .push (stmt p)? locals
  (.expect (&TokenKind::Dedent) "expected the `where` bindings to end" p)?
  .skip_separators p
  (.expect (&TokenKind::Dedent) "expected the `where` clause to end" p)?
  -- Plan 61 leaf 4's three refusals, checked once the whole group is read —
  -- a cycle is not a property of any one entry, and the shadow check needs
  -- the equation's parameters, which only a caller has.
  (where_group (&locals) params)?
  Ok locals
'''
NEW['parse/parser/stmt'] = r'''import crate::ParseError
import crate::parse::parser::Parser
import crate::parse::parser::assign_tail::assign_tail
import crate::parse::parser::control_flow::control_flow
import crate::parse::parser::local_test::local_test
import crate::parse::parser::parse_expr::parse_expr
import crate::parse::parser::starts_a_declaration::starts_a_declaration
import crate::parse::parser::try_await::try_await
import voltaic_ast::{Expr, Stmt}
import voltaic_lexer::{Keyword, TokenKind}

-- | One block element — `Binding | "return" Expr? | … | Expr`, closed by a
-- | `Newline`.
-- |
-- | Every production here builds an [`Expr`]: plan 55 leaf 1 deleted
-- | `StmtKind`, so what used to be "a statement kind" is one of the forms
-- | `Expr` gained, and the [`Stmt`] this returns is the trivia wrapper
-- | around it.
export(super) stmt : &mut Parser -> Result<Stmt, ParseError>
stmt p =
  leading = .peek_leading p
  start = (.peek_span p).0
  -- Plan 55 §1.5's carve-out, asked before anything is read: `test` is the
  -- one declaration a body may not hold, because rustc does not collect a
  -- nested `#[test]`. `RDX0112` rather than a lowering — see
  -- [`local_test`](local_test::local_test).
  (local_test p)?
  -- Plan 55 leaf 5: a declaration is a block element. Asked before control
  -- flow because every declaration keyword is reserved and none of them can
  -- begin an expression or a control form, so this costs one comparison and
  -- cannot shadow anything.
  --
  -- `item` is reused rather than reimplemented, so a local `data` and a
  -- module-scope one are the same parse and cannot drift — visibility,
  -- `deriving`, generics, the lot.
  if starts_a_declaration p
    declared = (.item p)?
    end = (.prev_span p).1
    -- **No separator is demanded**, unlike every other statement. `item`
    -- leaves the stream ready for the next one — which is why
    -- `items_until_dedent` calls `skip_separators` *before* each item and
    -- not after — so a single-line `const K : i32 = 2` has already taken
    -- its `Newline`, and asking for one again read the next statement's
    -- first token. A multi-line declaration ends on a `Dedent` instead,
    -- which is why the two forms behaved differently and why the one-line
    -- one was the failure.
    trailing = if .check (&TokenKind::Newline) p
      taken = .clone (.peek p).trailing
      .advance p
      taken
    else
      None
    -- `leading` is **empty, deliberately.** `peek_leading` peeks rather
    -- than consumes, so `Parser::item` captured this same trivia a moment
    -- ago and the declaration owns it — `emit_data`, `emit_import` and the
    -- rest have always printed an item's leading comments, because a
    -- module-scope declaration needs them. Keeping a copy here printed
    -- every comment in a local declaration **twice**.
    --
    -- Duplicated comments are still present, so nothing that checks for
    -- *loss* notices; `generated_rust_is_rustfmt_clean` is what caught it.
    wrapped = Expr::Item { item = Box::new declared, span = (start, end) }
    return Ok (Stmt { expr = wrapped, span = (start, end), leading = Vec::new (), trailing })
  -- Plan 20's control flow comes first: every one of its forms begins with
  -- a keyword or a label, none of which can begin an expression, so asking
  -- here costs one comparison and no backtracking.
  flow = (control_flow p)?
  if flow is Some found
    end = (.prev_span p).1
    trailing = (.newline_with_trailing "expected a line break after the statement" p)?
    return Ok (Stmt { expr = found, span = (start, end), leading, trailing })
  expr = if .check (&TokenKind::Kw (Keyword::Return)) p
    .advance p
    value = if .at_statement_end p
      None
    else
      Some (Box::new (parse_expr p 0)?)
    Expr::Return { value, span = (start, (.prev_span p).1) }
  else if (.try_draw p)? is Some drawn
    drawn
  else if (try_await p)? is Some awaited
    -- Doc 30's await-bind, asked **before** the binding: both can open with
    -- a pattern, and the two are told apart by which token ends it — `<-`
    -- or `=`. `try_await`'s scan stops at an `=` at depth zero for exactly
    -- that reason, so neither claims the other's statement.
    awaited
  else if (.try_binding p)? is Some bound
    Expr::Binding (Box::new bound)
  else
    value = (parse_expr p 0)?
    -- Assignment, read after the place expression because every one of its
    -- six tokens can only follow one. `assign_tail` has the detail, and a
    -- `case` arm body asks it too since plan 55 leaf 3.
    (assign_tail p value start)?
  end = (.prev_span p).1
  trailing = (.newline_with_trailing "expected a line break after the statement" p)?
  Ok (Stmt { expr, span = (start, end), leading, trailing })
'''

for rel, text in NEW.items():
    io.open(ROOT + rel + '.sdx', 'w', encoding='utf-8').write(text.lstrip('\n'))
    print('  created %s.sdx' % rel)

# **Eight translated imports are now one module over.** The head `port.sh`
# keeps is the pinned Rust's `use` list verbatim — which is right — and the
# three functions that used these went with them, so `unused_imports` would
# be an error under `-D warnings`.
MOVED = [
    'import crate::parse::parser::assign_tail::assign_tail',
    'import crate::parse::parser::control_flow::control_flow',
    'import crate::parse::parser::local_test::local_test',
    'import crate::parse::parser::starts_a_declaration::starts_a_declaration',
    'import crate::parse::parser::try_await::try_await',
    'import crate::parse::parser::unit_tail::unit_tail',
    'import crate::parse::parser::where_entry_is_a_function::where_entry_is_a_function',
    'import crate::parse::parser::where_group::where_group',
]
path = ROOT + 'parse/parser.sdx'
kept = [l for l in io.open(path, encoding='utf-8').read().split('\n')
        if l not in MOVED]
io.open(path, 'w', encoding='utf-8').write('\n'.join(kept))
print('  narrowed parse/parser.sdx by %d imports' % len(MOVED))

print('plan 82 batch 12: four files')
