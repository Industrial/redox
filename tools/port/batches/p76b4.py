# -*- coding: utf-8 -*-
"""Plan 76 batch 4: the lint pass.

**It patches batch 3's script, so the fix lands on the *next* build.** A batch
that edits another batch runs after that batch has already executed, which is
the non-idempotence plan 73 recorded for `p73b1fix`: the build has to be run
twice, and the second run is the one that counts.

Two source fixes and two allows.

`f64::from` is passed by name rather than wrapped in a lambda, which Voltaic has
supported since plan 57's methods-as-functions and clippy asks for.

`while_let_loop` joins `matches!` and `single_match` as **a lint about a Rust
form Voltaic cannot write**: `while let Some(at) = rest.find('%')` has no
spelling, so `alpha_renamed` is a `loop` with a `case` whose `None` arm breaks.
Three lints now say the same thing — Voltaic cannot pass a pattern — and this is
the first crate to need the third.

And `no_effect` / `unused_unit`, for the sixth crate. Plan 75 §2.4 already
called repeating the paragraph the wrong answer; this records that it has now
happened once more.
"""
import io

# --- the closure ---------------------------------------------------------
P = '/home/tom/Code/rust/voltaic/tools/port/batches/p76b3.py'
s = io.open(P, encoding='utf-8').read()
OLD = '(.map (\\\\n -> f64::from n) (.ok (u32::try_from total_bytes)))'
NEW = '(.map f64::from (.ok (u32::try_from total_bytes)))'
if OLD in s:
    io.open(P, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
    print('p76b3: f64::from by name')
else:
    assert NEW in s, 'p76b3: neither spelling'
    print('p76b3: already by name')

# --- the allows ----------------------------------------------------------
P = '/home/tom/Code/rust/voltaic/crates/voltaic-bench/sdx/lib.sdx'
WHY = {
    'while_let_loop': (
        "-- The third lint about a Rust form Voltaic cannot write, after\n"
        "-- `match_like_matches_macro` and `single_match`: `while let Some(x) = …`\n"
        "-- binds a **pattern**, and Voltaic cannot pass one. `alpha_renamed` is a\n"
        "-- `loop` with a `case` whose `None` arm breaks, which says the same thing\n"
        "-- in the constructs the language has.\n"),
    'no_effect': (
        "-- Plan 55 §1.6 requires a `()`-returning body to **end with `()`** rather\n"
        "-- than implying it, so a body ending in a loop or an assignment cannot\n"
        "-- become `()` by arriving last. clippy reads that `()` as a statement with\n"
        "-- no effect, and names it `unused_unit` where it closes a body.\n"
        "--\n"
        "-- The **sixth** crate to say so. Plan 75 §2.4 already called repeating\n"
        "-- this paragraph the wrong answer; it has now happened once more, and the\n"
        "-- decision is owed either a workspace-level allow or a change to §1.6.\n"),
    'unused_unit': None,
}

s = io.open(P, encoding='utf-8').read()
prefix = ''
added = 0
for lint in ('while_let_loop', 'no_effect', 'unused_unit'):
    line = '#![allow(clippy::%s)]' % lint
    if line in s:
        continue
    why = WHY[lint]
    prefix += (why if why else '') + line + '\n'
    if why:
        prefix = prefix[:-len(line) - 1] + line + '\n\n'
    added += 1
if added:
    io.open(P, 'w', encoding='utf-8').write(prefix + s)
    print('lib.sdx: %d allow(s) added' % added)
else:
    print('lib.sdx: the allows are already present')
