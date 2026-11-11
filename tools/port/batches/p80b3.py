# -*- coding: utf-8 -*-
"""Plan 80 batch 3: eight more of `redox-driver`'s bodies.

`write_outputs` is the first body to **read a field the translator renamed**:
`Cached.rust` is `r#rust` in the `.sdx`, because `rust` is a Redox keyword, so
the access is `content.r#rust`. That is the keyword rule reaching a *use* site
rather than a declaration, which is new — the five earlier collisions were all
declarations.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lint/allow_pragma', '''
export(crate) allow_pragma : &Trivia -> Lint -> bool
allow_pragma trivium lint =
  attribute_pragma trivium == Some lint || comment_pragma trivium == Some lint
''')

write('lint/suppressed_spans', '''
export(crate) suppressed_spans : &Program -> Lint -> Vec<Span>
suppressed_spans program lint =
  mut walk = PragmaWalk { lint, found = Vec::new () }
  walk_program program (&mut walk)
  walk.found
''')

write('lint/walk_stmts', '''
export(crate) walk_stmts : &[Stmt] -> &mut dyn ProgramWalk -> ()
walk_stmts stmts walk =
  .stmts stmts walk

  for stmt in stmts
    for nested in .statement_blocks stmt.expr
      walk_stmts (&nested.stmts) walk
  ()
''')

write('lint/binding_name', '''
export(crate) binding_name : &Binding -> Option<&Ident>
binding_name binding = case &binding.pattern of
  Pattern::Binding ident -> Some ident
  -- Every variant spelled out, with no wildcard: a pattern kind added to the
  -- AST must come back here and say whether it binds one name.
  Pattern::Literal { .. } or Pattern::Wildcard _ or Pattern::Range { .. } or Pattern::Apply { .. } or Pattern::Tuple { .. } or Pattern::Slice { .. } or Pattern::Record { .. } or Pattern::As { .. } or Pattern::Rest _ or Pattern::NamedRest { .. } or Pattern::Or _ -> None
''')

write('lint/comment_pragma_message', '''
export(crate) comment_pragma_message : Lint -> String
comment_pragma_message lint =
  -- Bound, because a call inside `#{…}` is one more thing for the
  -- interpolation scanner to be right about than it needs to be.
  comment = .allow_comment lint
  attribute = .allow_attribute lint
  format! \\-> "`-- #{comment}` is deprecated — write `\\#[#{attribute}]` on the line above instead (plan 53 §1.3)"

-- | Plan 53's wording, unchanged: the recorded `.stderr` compares as exact
-- | text, so parametrising must not move a character of it.
test "the unwrap wording is plan 53's"
  assert_eq! \\-> comment_pragma_message Lint::Unwrap \\-> "`-- redox: allow unwrap` is deprecated — write `\\#[redox::allow(unwrap)]` on the line above instead (plan 53 §1.3)"

-- | The deprecation names the lint it is about, which is why the message
-- | could not stay a constant.
test "each lint gets its own spellings"
  text = comment_pragma_message Lint::Shadow
  assert! \\-> .contains "-- redox: allow shadow" text
  assert! \\-> .contains "\\#[redox::allow(shadow)]" text
''')

write('diagnose/io_diagnostic', '''
export(crate) io_diagnostic : &str -> String -> Diagnostic
io_diagnostic path message =
  Diagnostic { code = DiagnosticCode::Foreign (.to_string "io"), severity = redox_diag::Severity::Error, message, span = (0, 0), help = None, path = .to_string path, source = String::new () }

test "renders without a snippet"
  d = io_diagnostic "a.sdx" (.to_string "cannot read a.sdx: missing")
  assert_eq! \\-> .text d.code \\-> "io"
  assert_eq! \\-> d.path \\-> "a.sdx"
  -- An empty source means the renderer has no snippet to draw, which it must
  -- survive rather than panic on.
  out = redox_diag::render (std::slice::from_ref (&d)) true
  assert! \\-> .contains "cannot read" out \\-> "#{out}"
  assert! \\-> !.contains "  |" out \\-> "no snippet should be drawn:\\n#{out}"
''')

write('rustc_path/is_real_rustc', '''
export(crate) is_real_rustc : &Path -> bool
is_real_rustc candidate =
  if !.is_file candidate
    return false
  probed = Command::new candidate |> .arg "--version" |> .output
  .is_ok_and (\\o -> .success o.status && .starts_with "rustc " (String::from_utf8_lossy (&o.stdout))) probed

-- | A shell exits zero for `--version` and prints something that is not a
-- | rustc banner, which is precisely the shape of the shims that caused this
-- | function to exist.
test "rejects a program that is not rustc"
  assert! \\-> !is_real_rustc (Path::new "/bin/sh")

test "rejects a path that does not exist"
  assert! \\-> !is_real_rustc (Path::new "/nonexistent/rustc")

test "rejects a directory"
  assert! \\-> !is_real_rustc (Path::new "/tmp")
''')

write('compile_dir/compile_file/write_outputs', '''
export(crate) write_outputs : &Path -> &Path -> &str -> &Cached -> Result<PathBuf, Vec<Diagnostic>>
write_outputs source out path content =
  stem = .map_or_else (\\-> .to_string "generated") (\\s -> .to_string (.to_string_lossy s)) (.file_stem source)
  target = .join (format! \\-> "#{stem}.rs") out

  out_shown = .to_string (.display out)
  (.map_err (\\e -> vec! \\-> (io_diagnostic path (format! \\-> "cannot create #{out_shown}: #{e}"))) (std::fs::create_dir_all out))?
  target_shown = .to_string (.display target)
  -- `content.r#rust`, because `rust` is a Redox keyword and the translator
  -- renamed the field. The first collision to reach a *use* site.
  (.map_err (\\e -> vec! \\-> (io_diagnostic path (format! \\-> "cannot write #{target_shown}: #{e}"))) (std::fs::write (&target) (&content.r#rust)))?

  mut map_name = .to_os_string (.expect "a name" (.file_name target))
  map_name |> .push ".map"
  sidecar = .with_file_name map_name target
  sidecar_shown = .to_string (.display sidecar)
  (.map_err (\\e -> vec! \\-> (io_diagnostic path (format! \\-> "cannot write #{sidecar_shown}: #{e}"))) (std::fs::write (&sidecar) (&content.map)))?

  Ok target
''')

print('plan 80 batch 3: eight files')
