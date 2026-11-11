# `tools/port` — the Rust-to-Redox porting aid

**A one-time aid per crate, not part of the compiler.** Row 72 set up the
self-hosting bootstrap and gave it ten successor rows, one crate each; this is
what those rows are driven by. Four documents name it, which is why it is
tracked rather than left in a scratch directory — the reproduction a plan
promises has to exist in the repository.

| | |
|---|---|
| `translate.py` | Rust to a partial `.sdx`: declarations, doc comments, imports, module heads. A body it cannot express gets a `-- TODO plan 68` marker. |
| `write.py` | Writes a hand-written body into a `.sdx`, merging the imports and keeping the translated head. |
| `port.sh` | Drives one crate: restore the pinned Rust, translate, apply the batches, format, and report any file that does not parse. |
| `batches/` | The hand-written bodies, per crate, in the order `port.sh` applies them. |

```bash
CRATE=redox-cli PIN=<sha> BATCHES="p79b1 p79b2" bash tools/port/port.sh
```

`port.sh`'s own header carries the five rules the rows have had to learn, each
of them twice: the input is a pinned commit's Rust and not the working tree's;
`git checkout <sha> -- <path>` stages what it writes; every batch states a
destination rather than a diff; a batch that patches another batch lands one
build late *and* must tolerate having already run; and the order afterwards is
**build → bless → seed-refresh**.

## This is not reproducible from a clean clone, and does not need to be

`translate.py` reads the **hand-written** Rust, which each port's own commit
untracks. So a batch set reproduces its crate only from the pinned commit named
in `port.sh`, and once a crate is ported its batches are history rather than a
build step. What *is* reproducible from a clean clone is the thing that
matters: `./bootstrap.sh` regenerates every ported crate's `src/` from its
committed `.sdx` using `seed/redox1.wasm`, and `crates/redox-cli/tests/port.rs`
checks that the `.sdx` and the generated Rust still agree.
