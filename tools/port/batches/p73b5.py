# -*- coding: utf-8 -*-
"""voltaic-diag batch 5 — reportable and source/map.

`reportable.sdx` is the crate's hardest file and needed two things that did not
exist before this batch:

* **`'_` in a type**, now that `type_vars` no longer declares it. Three
  `Formatter<'_>` and two `Box<dyn Display + '_>` here, plus
  `dyn Iterator<Item = …> + '_`.
* **An aliased import to break a name collision.** The class is
  `miette::Diagnostic` and a path cannot head an instance, but importing it
  bare would collide with `crate::Diagnostic`, which this file also uses.
  `import miette::Diagnostic as MietteDiagnostic` resolves both.

`diagnostic.sdx` goes back to the idiomatic `Formatter<'_>` it was written
with, the elision having been a workaround for a defect that is now fixed.

Neither body re-declares its type: the translator produced both
`data Diagnostic` and `data Reportable` above their `-- TODO` markers, and
a body that declares one again is `E0428`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

BODIES = {
    'diagnostic': '''import std::fmt::Display
import std::fmt::Formatter
import std::fmt::Result
import std::error::Error

instance Display Diagnostic
  fmt : &self -> &mut Formatter<'_> -> Result
  fmt self f = write! \\-> f \\-> "#{self.message}"

instance Error Diagnostic
''',

    'render/graphical_one/reportable': '''-- `Debug` as a **trait**, not the
-- derive macro of the same name: `instance Debug Reportable` emits
-- `impl Debug for Reportable`, and without this import `Debug` resolves to
-- the macro — `E0404: expected trait, found derive macro`.
import std::fmt::Debug
import crate::Diagnostic
import crate::render::anchor_span::anchor_span
import std::fmt::Display
import std::fmt::Formatter
import std::fmt::Result
import std::error::Error
-- Aliased, because the class is `miette::Diagnostic` and a **path cannot head
-- an instance** — the head is a run of type identifiers. Importing it bare
-- would collide with `crate::Diagnostic`, which this file also names.
import miette::Diagnostic as MietteDiagnostic

instance Reportable
  -- | Build the `miette` view of one diagnostic.
  -- |
  -- | Everything is cloned rather than borrowed: `miette::NamedSource` owns its
  -- | text, and the `Report` this ends up inside outlives the borrow a renderer
  -- | could lend it.
  export(crate) new : &Diagnostic -> Self
  new d = Reportable {
    message = .clone d.message,
    code = .text d.code,
    help = .clone d.help,
    span = anchor_span (&d.source) d.span,
    source = .with_language "Rust" (miette::NamedSource::new (&d.path) (.clone d.source)),
    severity = d.severity,
  }

instance Display Reportable
  fmt : &self -> &mut Formatter<'_> -> Result
  fmt self f = write! \\-> f \\-> "#{self.message}"

instance Debug Reportable
  fmt : &self -> &mut Formatter<'_> -> Result
  fmt self f = write! \\-> f \\-> "#{self.message}"

instance Error Reportable

instance MietteDiagnostic Reportable
  code : &self -> Option<Box<dyn Display + '_>>
  code self = Some (Box::new (&self.code))

  -- | What decides whether the rendered line reads `error[RDX…]` or
  -- | `warning[RDX…]` — doc 03 §12.7, plan 38.
  severity : &self -> Option<miette::Severity>
  severity self = Some (.to_miette self.severity)

  help : &self -> Option<Box<dyn Display + '_>>
  help self = self.help
    |> .as_ref
    |> .map (\\h -> (Box::new h) : Box<dyn Display>)

  source_code : &self -> Option<&dyn miette::SourceCode>
  source_code self = Some (&self.source)

  labels : &self -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>>
  labels self =
    -- A zero-width span would render no caret at all, so widen it to one byte.
    len = .max 1 (.saturating_sub self.span.0 self.span.1)
    Some (Box::new (std::iter::once (miette::LabeledSpan::new (Some (.to_string "here")) self.span.0 len)))
''',
}

for rel, text in BODIES.items():
    print('%-40s written' % write(rel, text))
