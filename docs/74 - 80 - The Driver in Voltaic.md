# Plan 80 — The Driver in Voltaic

`voltaic-driver`'s 48 files, 2,981 lines — the largest port so far, and the first
whose `.sdx` tree has **more** files than the Rust it replaces.

It found two compiler defects, both of the shape this row and plan 79 keep
meeting: **a rule applied where it cannot see enough context.** One of them doc
69 had already found, recorded as a surface decision, and deferred to a row —
and its own pinning contrast turns out to be wrong.

It also discharges the translator debt owed since row 75, which had been hit
seven times by the time this crate reached it.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/voltaic-driver/sdx/**` — **49** files, 2,489 lines |
| Generated | `crates/voltaic-driver/src/**`, 3,250 lines, untracked from this commit |
| Tests | 69, unchanged |
| List | `seed/PORTED` gains `voltaic-driver 80` |
| Seed | refreshed, 924,941 → 925,681 bytes |

**Forty-nine `.sdx` for forty-eight `.rs`**, and the extra file is the rule
working: `ScratchDir::new` declares its counter as a `static` *inside the
function*, Voltaic's `const`/`static` is module-scope (doc 03 §13.4), and
`one_construct_per_file.rs` then reported `scratch_dir.rs` declaring two
constructs. So the counter is `check/scratch_dir/next.sdx` and `scratch_dir`
becomes a module head. §2.5 has the two `#[cfg(test)]` counters that needed no
such move, and why.

Ten crates are now Voltaic — 297 files.

---

## 2 What it found

### 2.1 A postfix step crossed a dedent, which is doc 69's deferred row

`case … of` followed by a line opening with `.` — which is how every
method-call statement in Voltaic opens — read as a *field access on the `case`*,
and `application` then refused it as `RDX0115`:

```text
case hit of
  true -> ()
  false -> ()
.push_str "z" (&mut out)
->  RDX0115: method-call syntax is retired
```

`application` has guarded its **argument** loop on `just_dedented` since doc 18
§2.3 arrived — its own comment says `case … of` is *"the first expression with
layout of its own"* and that without the guard `label age = case age of …`
swallowed the next item's name. `postfix` had no such guard, and it runs inside
`atom`, below the place that does.

So `postfix` now refuses no step across a dedent, which is the same sentence
`application` already carried, and the fix is four lines.

**Doc 69 §10.3 found this and its third statement of it was wrong.** That
section is a careful piece of work: it records two narrower statements of the
finding and refutes both, by counterexample, before settling on *"it is the
**binding** path … a block in statement position never reaches `postfix`"*. The
settled statement is true of `if` and **false of `case`** — a `case` statement
goes through the expression path, so the shape above has no binding anywhere.
That is the same kind of counterexample §10.3 used against its own second
statement, found the same way: by running the input.

The rule was therefore never binding-versus-statement; it is wherever the
expression path runs. §12 of doc 69 owed the question to a row — *"the fix is a
parse decision about whether a `Dedent` terminates a binding, which wants a
row"* — and that framing is the one the correction replaces. The row is this
one; `tests/ui/block_valued_binding_eats_the_next_statement` is retired,
because its verdict was the defect, and all four shapes are pinned in
`crates/voltaic-parser/tests/parse.rs`.

### 2.2 `RDX0115`'s paren half, finished

Plan 79 §2.1 made the paren half require **adjacency**, so `postfix` refuses
`x.f(a)` and `application` refuses every spaced form. That fix compared spans;
it is now `!p.peek().ws_before`, which is how the `?.` check three lines above
it already says the same thing. Same behaviour, one idiom.

### 2.3 The translator prefixes `r#` itself, at last

Seven keyword collisions had been met one at a time by seven rows: `data` (row
75), `rust` (76), `class` and `data` again (78), and in this crate `with` as a
**module** name, `rust` as a field again, `of` in a path, and `errors` as a
pattern binder. Each arrived as `RDX0103` naming a line whose author chose the
name freely in Rust, and each fix was the same three characters. Row 75 owed
the translator the job and rows 76 and 78 restated the debt.

`translate.py` now reads the **57 keywords out of the lexer** rather than
retyping them, unions them with Rust's own reserved words — row 78's `final`
runs the other way, reserved in Rust and not in Voltaic — excludes the four that
cannot be raw identifiers, and prefixes every module name, field name and
record-variant field it emits. `module with` and `r#rust` landed with no
intervention.

**What it does not cover is use sites**, because the translator does not emit
them: a hand-written body naming `Fingerprint::of` or binding `Err errors`
still has to say `r#of` and pick another binder. Two of this crate's seven
collisions were of that kind, and `rust = …`'s diagnostic is still *"expected a
`"""` block of Rust after `rust`"*, which row 78 called overdue and remains so.

### 2.4 A macro argument that is a value gets brace-doubled

`macro_argument` brace-doubles every string literal reaching a macro, and
discloses the cost: *"a macro that wants a `{}` to reach it as a hole cannot be
written this way"*, with the reassurance that *"a string with no brace in it is
unaffected, which is every string in the corpus"*.

**The disclosed cost is the wrong way round, and the corpus claim is now
false.** `assert_eq! \-> &SRC[12 .. 13] \-> "}"` emits

```rust
assert_eq!(&SRC[12..13], "}}");
```

— a *value* comparison against two braces. It compiles, and only the assertion
failing says so. The template-position case the comment describes is the one
that works; the value-position case is the one that silently changes the
program.

Position alone cannot settle it (`assert_eq!("a {}", x)` compares a value;
`assert_eq!(a, b, "a {}")` is a template), so the fix wants the macro's name:
`format!`/`println!`/`panic!` take a template at argument 0,
`write!`/`assert!` at 1, `assert_eq!`/`assert_ne!` at 2, and a macro the
emitter does not know should double nothing — which is also the right answer
for `vec! \-> "a {} b"`, wrong today. That is a row, not a line in a port, so
this one asserts the character instead:

```text
assert_eq! \-> .next (.chars SRC[12 .. 13]) \-> Some '}'
```

A `char` literal is not a format template and is passed through, so the
assertion keeps its force. **Owed to a successor**, with the diagnosis above.

### 2.5 Smaller things

| | |
|---|---|
| **The first `class` declarations in a port** | `ProgramWalk` and `CacheReporter`. `examples/class.sdx` has the shape, and an `instance` of a class declared in **another file** must restate each signature — `RDX0101` asks for it, and `CacheReporter`'s instance needs none because its class is directly above. |
| **A block as an expression** | `ScratchDir`'s test scopes a `let` in `{ … }` so the `Drop` runs before the assertion. Plan 55 leaf 7 retired `do` rather than the construct, so an indented block after `=` *is* the expression. |
| **Two `#[cfg(test)]` counters stayed put** | `cache/store` and `compile_dir/sdx_in` hoist a `static` too, and `one_construct_per_file` does not count a `#[cfg(test)]` declaration — plan 79 taught it that, and this is the distinction earning its keep: a test's helper is not part of the file's surface. |
| **A second `write()` must carry the declaration** | The first consumed the `-- TODO` marker, so `write.py` then cuts at the first *construct* — the type — and a body holding only the impls drops it. `E0425: cannot find type ScratchDir`. |
| **`as` cast, the crate's one `CONFORMANCE` entry** | `(source.len() as u64).to_le_bytes()` is `u64::try_from` plus `unwrap_or u64::MAX`, the treatment plan 76 gave `as f64`. The fallback cannot mislead: a length that does not fit a `u64` cannot exist. |
| **A tuple binder in a `for`** | `for (n, text) in …` binds the pair and projects. Four more tuple-destructuring sites besides: two `mpsc`-style pairs and `Store::paths`' two callers. |
| **`\|` in a `filter_map`'s `match`** | The first multi-line lambda holding a `case`, with the closing parenthesis on the `_ -> None` arm. |
| **`.into()` on an unannotated lambda parameter is `E0282`** | A closure body is checked before any call site is seen, so `lint`'s `warning` closure takes a `DiagnosticCode` and each call site names the conversion. The general rule: when an annotation would have carried the inference, move the work to where the type is known. |
| **An array literal argument keeps its parentheses** | `.args ["--edition", …]` is an *index* on the method reference — `[` is not on `starts_argument`'s list, which `stands_alone`'s doc comment explains. |
| **`\#` does not escape in a comment** | Comments are trivia and pass through verbatim, so `-- | …\#[allow(unwrap)]…` reaches the `///` with the backslash. Caught by the doc-comment fidelity check, not by the gate. |

### 2.6 The receiver needs no `&`, for the second crate running

Ninety borrows — 43 in the library, 47 in the tests — were written `(&x)` in
receiver position and reported by `needless_borrow`. Plan 79 made the same
mistake wholesale and fixed it the same way.

It is worth stating as a rule rather than a correction, because the port writes
thousands of these: **a receiver is auto-referenced, so `.len found` is right
and `.len (&found)` is a lint.** The `&` belongs in an *argument* position,
where the callee's signature asks for a reference and nothing inserts it.

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 48 files are Voltaic, in 49 `.sdx` | yes |
| 2 | `voltaic-driver` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/voltaic-driver/src/` is gitignored and out of the index | yes |
| 4 | `crates/voltaic-driver/sdx/` **is** in the index | yes |
| 5 | Every test survives | yes — 69 before, 69 after, same names |
| 6 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 7 | A clean clone bootstraps it | yes |
| 8 | `stage1 == stage2` still holds | yes |
| 9 | No doc comment lost | yes — 1,093 → 1,127 lines, none missing |
| 10 | A postfix step cannot cross a dedent, all four shapes pinned | yes |
| 11 | The translator prefixes `r#` for every keyword it can see | yes |

---

## 4 Next

Two crates, 14,460 lines: `voltaic-emit` (6,203) and `voltaic-parser` (8,257).
Neither inherits the workspace allows yet, so each port's first `clippy` run
will say which of the five that crate needs.

Owed to a successor, from this row:

- **A macro's format-template argument should be known by name** (§2.4). A
  value argument with a brace in it is silently rewritten today.
- **A keyword collision needs its own diagnostic**, and the translator cannot
  reach a use site (§2.3). Owed since row 75.
- **The doc-comment fidelity check should be a test** — plan 79 §2.2 asked for
  it, and this row used it twice: once to find five reworded comments, once to
  find that `\#` does not escape in one.
- **`not` should be an operator or not a keyword** — plan 79 §2.5.
- **`FieldInit` needs a `leading`** — row 78.
