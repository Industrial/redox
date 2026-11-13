# -*- coding: utf-8 -*-
"""Batch 13: the crate-root allows, with the reason each one states.

Four lints, and none of them is a suppressed defect. Two are the same gap in
two directions — `matches!` and `if let` both take a **pattern**, which Voltaic
cannot pass — and two are plan 55 §1.6 disagreeing with clippy on purpose about
whether a unit body spells its `()`.

`voltaic-ast`, `voltaic-fmt` and `voltaic-diag` carry the first and third already, so
this is the fourth crate to state the same facts rather than the first.
"""
import io

P = '/home/tom/Code/rust/voltaic/crates/voltaic-lexer/sdx/lib.sdx'

ALLOWS = [
    ('match_like_matches_macro',
     "-- `matches!`'s second argument is a **pattern**, and Voltaic has no way to pass\n"
     "-- one — so a method asking *is this one of these variants* is a `case` with a\n"
     "-- `_ -> false` arm. Seven of them here: `TokenKind`'s two predicates,\n"
     "-- `TriviaKind::is_doc`, and four one-line tests inside the scanners.\n"
     "--\n"
     "-- `voltaic-ast`, `voltaic-fmt` and `voltaic-diag` carry the same allow for the same\n"
     "-- reason — plan 68 found it, and only a row making `matches!` reachable would\n"
     "-- retire it."),
    ('single_match',
     "-- The same gap one step over: Voltaic has no `if let`, so *do this if it\n"
     "-- matches* is a `case` with a `_ -> ()` arm. Three of them, each where the\n"
     "-- Rust wrote `if let Some(x) = …`."),
    ('no_effect',
     "-- Plan 55 §1.6 requires a `()`-returning body to **end with `()`** rather than\n"
     "-- implying it — deliberately, so that a body ending in a loop, a binding or an\n"
     "-- assignment cannot become `()` by arriving last. clippy reads that `()` as a\n"
     "-- statement with no effect. The two rules disagree by design, and §1.6's is\n"
     "-- the one with an argument behind it."),
    ('unused_unit',
     "-- The same `()`, in the places clippy names differently: `return ()` and a\n"
     "-- `()` closing a `case` arm."),
]

s = io.open(P, encoding='utf-8').read()
prefix = ''
added = 0
for lint, why in ALLOWS:
    line = '#![allow(clippy::%s)]' % lint
    if line in s:
        continue
    prefix += why + '\n' + line + '\n\n'
    added += 1
if added:
    io.open(P, 'w', encoding='utf-8').write(prefix + s)
    print('lib.sdx: %d allow(s) added' % added)
else:
    print('lib.sdx: the allows are already present')
