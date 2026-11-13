# -*- coding: utf-8 -*-
"""voltaic-diag batch 1 — the const, the two render entry points, the data type.

Three shapes established here and reused by every later batch:

* **`instance Trait Type`** works when the class name is imported **bare**.
  `instance fmt::Display Thing` does not parse — an instance head is a run of
  type identifiers, not a path. Neither ported crate had a single trait impl,
  so this was unexercised until now.
* **`&mut Formatter` with no lifetime.** `&mut Formatter<'_>` emits
  `fn fmt<'_>(…)`, hoisting the lifetime argument into the function's generic
  list, and rustc rejects it: *"`'_` cannot be used here: `'_` is a reserved
  lifetime name"* (E0637). The elided form is equivalent and compiles.
* **`#{x:?}` is Debug formatting** — emits `{:?}`. It exists, which neither
  plan 68 nor 69 needed to find out.

An instance with **no members** also works: `instance Error Thing` emits
`impl Error for Thing {}`.

**A body supplies only what the `-- TODO` marker says is owed.** Everything
above it is the translator's and is already in the file; re-declaring a
`module` or a `data` is `E0428`, and assuming less is there than actually is
loses it. This batch learned the first half the hard way, twice.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

BODIES = {
    'source/map/version': '''export const VERSION : u32 = 1
''',

    'render/graphical_one': '''import crate::Diagnostic
import crate::render::graphical_one::reportable::Reportable

-- | Render one diagnostic the way `miette` draws it: boxed, coloured, with the
-- | label pointing into a snippet of the real source.
-- |
-- | `miette`'s own entry point is `Debug`, not `Display` — the graphical
-- | handler is installed on the `Report`'s debug formatter — which is why this
-- | is a `#{…:?}` rather than a `.to_string`.
export(crate) graphical_one : &Diagnostic -> String
graphical_one d = format! \\-> "#{(miette::Report::new (Reportable::new d)):?}"
''',

    'render': '''import crate::Diagnostic
import crate::render::graphical_one::graphical_one
import crate::render::plain_one::plain_one

-- | Render every diagnostic, graphically or plainly.
-- |
-- | Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 4. One report per
-- | diagnostic, which is Emerald's `report_driver_error` loop — plan 03's
-- | two-level recovery already produces several errors and nothing yet
-- | displayed more than the first.
-- |
-- | `plain` exists for the UI tests, not for users: a corpus comparing bytes
-- | cannot depend on terminal width or colour detection. Making it a flag
-- | rather than an environment sniff keeps the test honest about what it
-- | asserts.
export render : &[Diagnostic] -> bool -> String
render diagnostics plain =
  mut out = String::new ()
  for pair in .enumerate (.iter diagnostics)
    -- One parameter and two bindings, where the Rust wrote `for (i, d)`:
    -- **`for` binds a name, not a pattern** (plan 68's gap), and a lambda
    -- parameter is a name too (plan 69's third finding).
    i = pair.0
    d = pair.1
    if i > 0
      .push '\\n' out
    if plain
      .push_str (&(plain_one d)) out
    else
      .push_str (&(graphical_one d)) out
  out
''',

    'diagnostic/severity': '''instance Severity
  -- | Whether a diagnostic of this severity should stop the build.
  export is_fatal : self -> bool
  is_fatal self = case self of
    Severity::Error -> true
    _ -> false

  -- | `miette`'s equivalent, which is what decides whether the rendered line
  -- | reads `error[RDX…]` or `warning[RDX…]`.
  export to_miette : self -> miette::Severity
  to_miette self = case self of
    Severity::Error -> miette::Severity::Error
    Severity::Warning -> miette::Severity::Warning
''',
}

for rel, text in BODIES.items():
    print('%-40s written' % write(rel, text))
