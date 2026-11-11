# Plan 77 — The MCP Server in Redox

`redox-mcp`'s 19 files, 808 lines. Row 72's fifth successor, and the first to
need **async**, a **bounded generic**, and a **derive path** — three surface
features that existed and could not be reached from where this crate needed
them.

It is also the row that fixed the formatter defect plan 69 recorded wrongly,
because that defect blocked the port outright.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/redox-mcp/sdx/**` — 19 files, 326 lines of Rust expressed |
| Generated | `crates/redox-mcp/src/**`, untracked from this commit |
| Tests | 22, unchanged |
| Seed | refreshed, 925,398 bytes |

---

## 2 What the surface had to reach

### 2.1 `deriving` could not take a path

`deriving (Debug, Clone, serde::Deserialize, schemars::JsonSchema)` was
*"expected `)` to close the `deriving` list"*: the clause read **one
identifier** per entry, and nine of this crate's nineteen files derive by
qualified name.

`#[derive(serde::Deserialize)]` is ordinary Rust, and `deriving` emits straight
into that attribute — it simply could not read what it writes. Importing the
derive macro and naming it bare would also work, but that is the *caller's*
spelling to choose, not the parser's to force. Segments are joined into the
`Ident`'s text, which the emitter writes verbatim, so nothing downstream
changed.

### 2.2 `async` and `<-` were already there, and `examples/async.sdx` is why

Nothing had to be added. Plan 39 wrote `examples/async.sdx` as a *running*
program precisely so these would be demonstrated rather than described, and it
answered both questions in one reading:

- a nullary `async fn` puts the marker before the **return** type —
  `noop : async ()`, so `main : async Result<(), Box<dyn Error>>`;
- `.await` is `<-` — `x <- e` binds, `<- e` awaits for effect only.

**An async method** needed one more step: the marker goes on the first type,
and `&self` would be first. Plan 67 lets the receiver sit anywhere, so
`redox_check : async Parameters<CheckRequest> -> &self -> Result<…>` puts the
marker on the parameter and `emit::signature` still writes the receiver first.

That is a row's worth of value from a nine-line example, and it is the argument
for `examples/` being runnable rather than illustrative.

### 2.3 A bounded generic function

`server_content : Serialize t => Result<t, String> -> …`, exactly as the same
example's `block_on : Future f => f -> f::Output`. The trait is imported and
named bare, because a constraint reads one name.

---

## 3 The formatter defect plan 69 recorded wrongly

`tokio::task::spawn_blocking(move || …)` appears four times, so the port needed
`move \-> …` — and plan 69 had recorded *"`redox fmt` diverges on a nullary
lambda in argument position, one parenthesis per pass, unbounded"*.

**That description is wrong in every part except where it pointed.** Measured:

```
pass 0   later = call (move \-> run held)
pass 1   later = call (\move  -> run held)
pass 2   later = call (\move  -> run held)
```

Nothing is unbounded — it is stable after one pass. No parenthesis is
involved. And it is not about the lambda being nullary: `move \x -> …` was
mangled the same way.

What it is: `lambda_text` opened its output with the backslash and pushed
`"move "` **after** it, putting the keyword inside the parameter list. `move`
is a keyword, so `\move  -> e` is `RDX0103: expected a lambda parameter name`
— **the formatter turned valid source into source its own parser rejects**,
which is plan 74 §3's class, and worse here because it is *stable* on the
broken output and so never corrects itself.

A nullary lambda is only the shape where the mangling has nowhere to hide:
with no parameter between the backslash and the arrow, the keyword lands where
a reader sees it.

**And a second defect underneath it.** With `move` fixed, `move \-> 1` came out
`move \ -> 1`: the separator always carried a leading space, which belongs to
the parameter list. No corpus file had ever shown it, because **no corpus file
had a true nullary lambda** — `name! \-> e` is doc 18's macro-call syntax and
takes `macro_call_text`'s path entirely, so every `\->` in the repository until
now was a macro and not a lambda at all.

Both are in `tests/fmt/expressions.sdx` now, as a fixture pair.

---

## 4 Three smaller findings

**`Parameters(request)` has no Redox spelling** in a parameter position — a
parameter is a bare name — so the whole value is taken and `params.0` reads it
out. `Parameters` is a tuple struct, so that is the same projection the pattern
performed.

**`rust` is a keyword as a *binding* name too**, and the diagnostic is the
wrong one: `generated = emitted.0` written as `rust = emitted.0` earns
`RDX0101: expected a """ block of Rust after rust` — a message about the escape
hatch, which is not what the author wrote. Three keyword collisions in three
crates now (`data`, a field called `rust`, a binding called `rust`), and
**a message naming the collision would have cost a minute instead of a
puzzle.**

**An attribute the head already carries must not be repeated.** `main.sdx`'s
body re-stated `#[tokio::main]`, so the generated file had two — and the outer
macro then saw the inner's expansion, a plain `fn main`, reporting *"the
`async` keyword is missing from the function declaration"*. The translated head
keeps every attribute above the `-- TODO` marker; a body adds only what the
head lacks.

---

## 5 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 19 files are Redox | yes |
| 2 | `redox-mcp` is in `seed/PORTED`, and `port.rs`'s five assertions cover it | yes |
| 3 | `crates/redox-mcp/src/` is gitignored and out of the index | yes |
| 4 | `crates/redox-mcp/sdx/` is in the index | yes |
| 5 | Every test survives | yes — 22 before, 22 after |
| 6 | `deriving` takes a path, with a test | yes |
| 7 | `move \-> e` round-trips through `redox fmt` | yes — pinned by a fixture pair |
| 8 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 9 | A clean clone bootstraps it | yes — seed refreshed |
| 10 | `stage1 == stage2` still holds | yes |

---

## 6 Owed to successor rows

- **A keyword collision deserves its own diagnostic** (§4). `rust = …` should
  say *"`rust` is a keyword; write `r#rust`"*, not report the escape hatch.
- **The translator could prefix `r#`** on a keyword-named field or parameter,
  carried from plan 76 §4.2.
- **Six crates carry the `no_effect` allow** — carried from plan 76 §6, and
  `redox-mcp` did not need it, so the count is unchanged.
- Everything plan 74 §9 lists, less the `move`-lambda entry, which this row
  closed.

---

## 7 Next

Five crates, roughly 19,700 lines: `redox-lsp` (934), `redox-cli` (1,376),
`redox-driver` (2,981), `redox-emit` (6,203), `redox-parser` (8,223).
