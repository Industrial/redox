# Plan 12 — Watcher

Plan ID: `watcher`
Depends on: 10 (`incremental-cache`), 11 (`cli-surface`)
Milestone: 3 — *the compiler is fast enough to live in an edit loop*
Status: `done`

---

## 1. What this plan is

`redox watch` — rebuild when a `.sdx` changes, and keep watching when it fails.

Doc 01 §28 bounds the feature: it watches and rebuilds, and it is **not a build system**.
It does not run tests, resolve dependencies, or sequence tasks. Doc 01 §1.3 forbids
replacing Cargo, and a watcher that grew a task graph would be doing exactly that.

### 1.1 The design that makes this plan small

Doc 04's row names the property: *"correct by cache-hit rather than by mtime heuristic."*

A watcher normally has to be clever. Filesystem events arrive in bursts — an editor writes
a temporary file, renames it, touches the directory — so a naive loop rebuilds four times
per save, and the usual fix is a debounce window, which is a timing heuristic that is
wrong on slow disks and on fast typists.

Plan 10's cache removes the need for a *timing* heuristic. **Every event triggers a
compile, and an unchanged file is a cache hit**, which costs a hash of the file instead of
a parse and an emit. Debouncing becomes an optimization with no correctness role, which is
why this plan has no timing constants in it.

That is the payoff for having built the cache first, and it is worth stating because the
reverse order would have produced a watcher full of tuning parameters.

> **This section originally went further and was wrong.** It claimed the watcher *"does not
> need to be correct about which events matter; it needs only to not miss any."* It does
> need to be correct: the build **reads its own input**, `inotify` reports that read as an
> event on a `.sdx`, and a path-only filter therefore made the compile its own trigger —
> 97,442 builds in six seconds. A cache makes a redundant compile cheap; it cannot bound
> how many there are. §10.1 records the measurement and the fix.

### 1.2 Surviving errors is the whole value

A watcher that exits on a compile error is worse than no watcher: the user fixes the typo
and nothing happens, because the process died three seconds ago. So the loop catches
everything a compile can produce — a lex error, a parse error, an unwritable output — prints
it, and waits for the next event.

This is also why plan 05's diagnostics had to come first. The loop prints a `Diagnostic`
through `redox_diag::render` and gets the same output `redox build` gives, rather than a
watcher-specific error format that would drift from it.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `watch` is declared but unimplemented | plan 11's table | `State::Planned { plan: 12, id: "watcher" }` — this plan is what the CLI already points at |
| `notify` is obtainable | `cargo add --dry-run` | `notify v8.2.2` |
| A compile can be driven without the CLI | plan 09 | `redox_driver::compile_dir` returns `Vec<Diagnostic>` |
| An unchanged file is cheap to recompile | plan 10 | Content-addressed hit, asserted by `cache.rs` |

### 2.1 `notify` is the third dependency

Doc 02 §5.5 requires a written reason. Watching a directory means `inotify` on Linux,
`kqueue` on the BSDs, `FSEvents` on macOS and `ReadDirectoryChangesW` on Windows — four
platform APIs behind one interface, which is precisely the library function doc 02 forbids
implementing. The alternative is polling `mtime` on a timer, which is the heuristic §1.1
exists to avoid and which scales with directory size rather than with edits.

---

## 3. Concrete proof this plan targets

A watcher is a timing-dependent thing, so the proof is split to keep it honest:

1. **The event filter is tested exhaustively, without the filesystem.** Which paths warrant
   a compile is pure logic and deserves pure tests.
2. **One end-to-end test actually watches**: start the watcher, write a `.sdx`, and assert
   the output appears. Bounded by a generous timeout, and skipped if no rustc — a real
   filesystem event through a real `notify` watcher, because the claim is about the OS.
3. **A compile error does not end the loop**: write a broken file, then a good one, and
   assert the second still builds. This is §1.2's property and the one most likely to
   regress.

---

## 4. Leaves

### Leaf 1 — `leaf-watch-target`

`watch_command/target.rs`. Resolve the argument to a file or a directory, and the directory
to watch. Watching a file's *parent* rather than the file is required: editors replace
files rather than writing them in place, so a watch on the inode is lost on the first save.

### Leaf 2 — `leaf-is-relevant`

`watch_command/is_relevant.rs`. Whether an event's *path* warrants a compile — a `.sdx`
changed, and not Redox's own `.xrs` output, and not one of the editor temporaries (`.swp`,
`~`, `4913`) that would otherwise cause a compile per keystroke in some editors.

`watch_command/is_change.rs`. Whether an event's *kind* is a change at all. Added after
§10.1: a path filter alone is not sufficient, because the build's own read of the source is
an event on a `.sdx`. Both filters are necessary and neither is sufficient.

### Leaf 3 — `leaf-watch-loop`

`watch_command.rs`. Build once at start, then compile on every relevant event, printing
diagnostics and continuing. No debounce, by §1.1.

### Leaf 4 — `leaf-proof-and-gate`

`redox-cli/tests/watch.rs` for §3, the gate, and the table entry flipped to `Ready`.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | `redox watch <file>` builds once immediately, then on each change |
| 2 | A compile error is printed and the loop continues |
| 3 | An editor temporary does not trigger a compile |
| 4 | A non-`.sdx` change does not trigger a compile |
| 5 | The watcher survives the watched file being replaced, not just modified |
| 6 | Plan 11's table reports `watch` as `Ready`, and `--help` no longer calls it planned |
| 7 | No timing constant decides correctness (§1.1) |
| 8 | Full gate green, `cargo audit` clean with `notify` |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Running tests on change | **rejected** | Doc 01 §28 and §1.3. `cargo watch -x test` already exists and composes |
| A task graph or dependency ordering | **rejected** | That is a build system |
| Debouncing | — | An optimization with no correctness role (§1.1). Worth adding if a profile ever shows the hash cost mattering, which at one file per save it will not |
| Watching a Cargo project through `build.rs` | — | `cargo build` reruns the script itself; `redox watch` is for the `redox build` loop, where there is no Cargo to do it |
| Clearing the screen between runs | — | A watcher that erases the error you were reading is a common and irritating default |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald has no watch mode. Doc 01 §28 is Redox's own requirement, and the design
in §1.1 depends on plan 10's cache, which Emerald's equivalent could not have been built
on — its cache is per-query over a compiler that produces object files, not a transpiler
whose unit of work is one file.

---

## 9. Decision log

- **Correctness comes from the cache, not from timing** (§1.1). No debounce window, no
  mtime comparison, no settle delay. Criterion 7 asserts the absence.
- **The parent directory is watched, not the file.** Editors replace rather than rewrite,
  and a watch on the file is lost on the first save — a bug that looks like "the watcher
  stopped working after I saved once".
- **Errors are printed and swallowed** (§1.2). A watcher that exits on a typo is worse than
  none.
- **Diagnostics go through `redox_diag::render`**, not a watcher-specific format, so the
  loop's output cannot drift from `redox build`'s.
- **Editor temporaries are filtered.** Not for correctness — the cache makes a spurious
  compile cheap — but because `[cache] a.sdx HIT` printed four times per keystroke is
  noise the user has to learn to ignore.
- **`notify` earns its place** (§2.1): four platform APIs behind one interface.
- **The screen is not cleared.** Erasing the diagnostic the user is reading is a default
  worth declining.

---

## 10. Result

`done`. 329 tests pass; `fmt`, `clippy -D warnings`, the full gate and `cargo audit` are
green. `watch` is `Ready` in plan 11's table, and `--help` no longer calls it planned.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Builds once, then on each change | `watch.rs`: `it_builds_at_start_and_on_change` — a real `notify` watcher on a real file |
| 2 | A compile error does not end the loop | `a_compile_error_does_not_end_the_loop`: break it, fix it, assert the fix still builds and the process lives |
| 3 | Editor temporaries do not trigger | `is_relevant.rs`: six named artefacts, including vim's `4913` and emacs' `.#` and `#…#` |
| 4 | A non-`.sdx` change does not trigger | `is_relevant.rs`, including `.xrs` and `.xrs.map` — Redox's own output |
| 5 | Survives the file being **replaced** | `it_survives_the_file_being_replaced`, renaming over the source twice |
| 6 | The CLI reports `watch` as ready | `the_cli_reports_watch_as_ready` |
| 7 | No timing constant decides correctness | `the_watcher_contains_no_debounce`, which reads the source for `sleep`/`Duration`/`Instant` |
| 8 | Gate and audit green | `moon run :check :lint :test :cargo-check :cargo-test :audit` |

### 10.1 §1.1 was wrong, and the measurement is why

This plan claimed, in §1.1:

> The watcher therefore does not need to be correct about *which* events matter; it needs
> only to not miss any.

That is **false**, and criterion 5's test is what exposed it. The first implementation
filtered events by *path* alone: a `.sdx` changed, so compile. Pointed at a real file it
produced

```text
builds: 97442
```

in six seconds, on a file nobody was editing. The instrumented loop says why in one line:

```text
EVENT Access(Open(Any)) ["/tmp/redox-pr2/main.sdx"]
```

**The build reads its own input, and `inotify` reports that read.** A `.sdx` path, so the
filter said yes, so the loop compiled, which opened the file, which was another event.

The reasoning error is worth naming precisely, because it is seductive: a cache makes a
*redundant* compile cheap, and from there it is one short step to "so redundant compiles
do not matter". They do not matter when something else bounds how many there are. Nothing
did. **Cheap multiplied by unbounded is still unbounded**, and the compile was its own
trigger.

The fix is `is_change`, which filters on event *kind*: `Access` is reading, not changing.
`Modify(Metadata)` is excluded for the same reason by a different door — under `relatime`
the build's own read updates `atime`, which would have reopened the loop.

Two filters, and the plan now says both are necessary: `is_change` rejects the build's read
of the source, `is_relevant` rejects the generated `.xrs` it writes. Either one alone
leaves a loop. After the fix the same probe reports `builds: 3` — one at start, one per
rename — with the correct final output.

What survives of §1.1 is the narrower and still useful claim: **no timing heuristic is
needed**, because the cache makes redundant work cheap once the event filter is correct.
There is no debounce, no settle delay and no mtime comparison, and criterion 7 asserts
their absence by reading the source.

### 10.2 Why the test found it and the design review did not

Criterion 5 — "survives the file being *replaced*, not just modified" — was in the plan
from the start, and the first version of the test used `fs::write`, which truncates in
place and leaves the inode alone. It passed. Writing `replace()` to do what an editor
actually does, a `rename` over the source, is what turned a passing suite into a 20-second
timeout and then into the trace above.

The lesson is about fidelity rather than coverage: the test was *present* and *wrong*,
because `fs::write` is not how files get saved. A watcher test that does not rename is
testing a filesystem operation no editor performs.

### 10.3 Disclosures

- **`Access` and `Modify(Metadata)` never trigger a rebuild.** A `touch` with no content
  change therefore does not rebuild. This is correct — there is nothing to rebuild — but it
  differs from `make`, and someone used to `touch`-to-force will find it surprising.
  Deleting the `.xrs` and saving is the way to force one.
- **`EventKind::Any` and `Other` do trigger**, on the reasoning that a missed edit looks
  like a broken watcher while an unknown-kind loop would require a `notify` bug. The
  trade-off is stated in `is_change` rather than left implicit, because §10.1 is what
  happens when a loop risk is not written down.
- **A directory target compiles every `.sdx` on every change**, not just the changed one.
  Plan 10's cache makes the others a hash each. Compiling only the changed file would be an
  optimization with a correctness question attached — what if a change makes another file
  invalid — and that question is not answerable until modules exist.
- **No debouncing**, by §1.1's surviving claim. A burst of events during one save produces
  a burst of cache hits, which is cheap and bounded by the burst.
- **The screen is not cleared** between runs, and `redox watch` writes only to stderr, so
  it composes with a pipe.
