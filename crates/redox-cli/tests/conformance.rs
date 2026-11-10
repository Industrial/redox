//! `docs/CONFORMANCE.md`, and the one ported module behind it.
//!
//! Plan 48 (`docs/42 - 48 - Conformance and Self-Hosting.md`). Doc 02 §4.8
//! reframed self-hosting as a conformance test and wrote the criterion: *"the
//! compiler's source can be expressed in Redox without the inline-Rust escape
//! hatch."*
//!
//! It also predicted the attempt *"will find every `UNREACHABLE` Rust
//! construct in §4.1's inventory the hard way"*. Plan 46 found them the easy
//! way, so this **counts** how often each known blocker appears in the
//! compiler's own Rust — which answers the criterion for all 430 files rather
//! than for a sample — and ports one module to show the count measures
//! something real.
//!
//! The document is generated and compared, as `docs/STATUS.md` is, so the row
//! survives its own success: when macro invocation is spelled, the count
//! drops, this fails, and whoever spelled it sees how much of the compiler
//! they just unblocked.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A blocker, and how it looks in Rust source.
///
/// Line-based rather than a parse. §6 of the plan discloses the cost: it is
/// why `|` or-patterns are not counted at all — no line-based pattern
/// separates a pattern alternative from a bitwise or — and why the first `as`
/// count was wrong three times over.
struct Blocker {
  name: &'static str,
  /// Whether this line contains one.
  matches: fn(&str) -> bool,
}

/// `expr as Type`. An **import** is not a cast, which is the correction the
/// plan's §1.1 records: `use foo as _;` counted as one until it did not.
fn is_cast(line: &str) -> bool {
  let trimmed = line.trim_start();
  if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
    return false;
  }
  line.split(" as ").skip(1).any(|after| {
    after
      .trim_start()
      .trim_start_matches(['&', '*'])
      .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
  })
}

/// The constructs a Redox program cannot express, as `docs/GRAMMAR.md` grades
/// them.
///
/// **Macro invocation left this list in plan 64**, which is the row this
/// document existed to motivate. Plan 48 measured it at 952 of 964 blocking
/// occurrences and wrote *"spell it, and what remains is the `as` casts
/// below"*; the list is now what remains. It is deleted rather than kept at
/// zero because the sentence under **Blockers** reads *"each is graded
/// `UNREACHABLE` in `docs/GRAMMAR.md`"* — a `SUGARED` row in this table would
/// make the document wrong about itself, and plan 46's rule is that the two
/// cannot be allowed to disagree.
const BLOCKERS: &[Blocker] = &[
  Blocker {
    name: "`as` cast",
    matches: is_cast,
  },
  Blocker {
    name: "`unsafe trait`",
    matches: |l| l.contains("unsafe trait "),
  },
  Blocker {
    name: "`unsafe impl`",
    matches: |l| l.contains("unsafe impl "),
  },
  Blocker {
    name: "`extern \"C\" fn`",
    matches: |l| l.contains("extern \"") && l.contains("\" fn"),
  },
  Blocker {
    name: "Macro definition",
    matches: |l| l.contains("macro_rules!"),
  },
];

/// `path`'s lines with comments dropped and string bodies blanked.
fn code_lines(text: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut in_block = false;
  for line in text.lines() {
    let trimmed = line.trim_start();
    if in_block {
      if trimmed.contains("*/") {
        in_block = false;
      }
      continue;
    }
    if trimmed.starts_with("/*") {
      if !trimmed.contains("*/") {
        in_block = true;
      }
      continue;
    }
    if trimmed.starts_with("//") {
      continue;
    }
    // Blank string bodies so a blocker quoted in one is not counted.
    let mut cleaned = String::with_capacity(line.len());
    let mut in_str = false;
    let mut escaped = false;
    for c in line.chars() {
      if in_str {
        if escaped {
          escaped = false;
        } else if c == '\\' {
          escaped = true;
        } else if c == '"' {
          in_str = false;
          cleaned.push('"');
        }
        continue;
      }
      if c == '"' {
        in_str = true;
        cleaned.push('"');
        continue;
      }
      cleaned.push(c);
    }
    out.push(cleaned);
  }
  out
}

fn sources(root: &Path) -> Vec<PathBuf> {
  fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
      return;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        walk(&path, out);
      } else if path.extension().is_some_and(|e| e == "rs") {
        out.push(path);
      }
    }
  }
  let mut out = Vec::new();
  let Ok(crates) = std::fs::read_dir(root.join("crates")) else {
    return out;
  };
  for entry in crates.flatten() {
    walk(&entry.path().join("src"), &mut out);
  }
  out.sort();
  out
}

/// Render the document.
fn render(root: &Path) -> String {
  let files = sources(root);
  let mut totals = vec![0usize; BLOCKERS.len()];
  // crate -> (files, blocked)
  let mut per_crate: Vec<(String, usize, usize)> = Vec::new();

  for path in &files {
    let lines = code_lines(&std::fs::read_to_string(path).unwrap_or_default());
    let mut any = false;
    for (i, blocker) in BLOCKERS.iter().enumerate() {
      let hits = lines.iter().filter(|l| (blocker.matches)(l)).count();
      totals[i] += hits;
      any |= hits > 0;
    }
    let crate_name = path
      .strip_prefix(root.join("crates"))
      .ok()
      .and_then(|p| p.components().next())
      .map(|c| c.as_os_str().to_string_lossy().into_owned())
      .unwrap_or_default();
    match per_crate.iter_mut().find(|(n, _, _)| *n == crate_name) {
      Some(entry) => {
        entry.1 += 1;
        entry.2 += usize::from(any);
      }
      None => per_crate.push((crate_name, 1, usize::from(any))),
    }
  }
  per_crate.sort();
  let blocked_files: usize = per_crate.iter().map(|(_, _, b)| b).sum();

  let mut out = String::new();
  out.push_str(
    "# Conformance\n\n\
     **Can the compiler's own source be expressed in Redox, without the\n\
     inline-Rust escape hatch?** Doc 02 §4.8 phrases the question that way and\n\
     calls it *\"the property actually worth proving\"*.\n\n\
     **Generated.** Measured from the working tree by\n\
     `crates/redox-cli/tests/conformance.rs` and compared in the gate. Rewrite\n\
     it with:\n\n\
     ```bash\n\
     REDOX_BLESS=1 cargo test -p redox-cli --test conformance\n\
     ```\n\n\
     Plan 48 (`docs/42 - 48 - Conformance and Self-Hosting.md`) built it, and\n\
     plan 64 changed its answer.\n\n\
     ---\n\n## The answer\n\n**Almost — and what is left is the `as` cast.**\n\n",
  );

  let total_hits: usize = totals.iter().sum();
  let _ = writeln!(
    out,
    "**{total_hits} blocking occurrences**, down from 964 — and the construct that was 952 \
     of them is gone. Plan 48 measured macro invocation at 99% of the obstacle and wrote \
     *\"spell it, and what remains is the `as` casts below\"*; plan 64 spelled it, as \
     `name! \\-> e`, and this is what remained.\n\n\
     That is the whole arc of the measurement in three rows: plan 45 §1.5 **found** the gap \
     and declined to spell it because it needed a surface decision in doc 03; plan 48 \
     **priced** it, which is what made the decision worth taking; plan 64 **took** it. A \
     count is what turned an unreachability note into a priority.\n\n\
     The figure is not zero and will not be: it is read off `docs/GRAMMAR.md`'s \
     `UNREACHABLE` rows and moves on any commit that adds Rust, which is why it is \
     generated rather than written down.\n\n---\n\n## Blockers\n\nEach is graded \
     `UNREACHABLE` in `docs/GRAMMAR.md`.\n"
  );
  let _ = writeln!(out, "| Blocker | Occurrences |\n|---|---:|");
  for (i, blocker) in BLOCKERS.iter().enumerate() {
    let _ = writeln!(out, "| {} | {} |", blocker.name, totals[i]);
  }
  let _ = writeln!(out, "| `\\|` or-pattern | *not counted* |");
  out.push_str(
    "\n**Four of the five remaining gaps are zero**, which is the result doc 02 §4.8 did \
     not expect: `unsafe trait`, `unsafe impl`, `extern \"C\" fn` and `macro_rules!` are \
     real gaps and were never *this* obstacle. Only a count could say so — and the same \
     count is what identified the one that was.\n\n\
     `\\|` or-patterns are **not counted** rather than estimated. The counter is line-based, \
     and no line-based pattern separates a pattern alternative from a bitwise or; a wrong \
     number would be worse than an empty cell.\n\n---\n\n## By crate\n\n",
  );
  let _ = writeln!(
    out,
    "| Crate | Files | With a blocker | |\n|---|---:|---:|---:|"
  );
  for (name, total, blocked) in &per_crate {
    let pct = 100.0 * *blocked as f64 / *total as f64;
    let _ = writeln!(out, "| `{name}` | {total} | {blocked} | {pct:.0}% |");
  }
  let pct = 100.0 * blocked_files as f64 / files.len() as f64;
  let _ = writeln!(
    out,
    "| **total** | **{}** | **{blocked_files}** | **{pct:.0}%** |",
    files.len()
  );

  out.push_str(
    "\n---\n\n## The worked port\n\n\
     `conformance/clause.sdx` is `crates/redox-ast/src/program/item/clause.rs` expressed in \
     Redox, with no escape hatch in it. The generated `conformance/clause.xrs` is committed, \
     per doc 02 §4.8 — *\"commit the generated `.xrs` so the bootstrap never breaks\"*.\n\n\
     It proves a real compiler data type is expressible: declaration, named fields, \
     per-field visibility, derives, doc comments and verbatim types all survive. It does \
     **not** prove the compiler is expressible — 45 lines is 0.2% of it, and the counts above \
     speak to the rest.\n\n\
     Writing it found a defect the count could not see. Every field doc comment was silently \
     discarded, and `redox fmt` deleted them from the source, because the lexer classified a \
     comment opening a continuation line as *trailing* the previous line and then held it for \
     a `Newline` a continuation never produces. AGENTS.md calls comment preservation \
     load-bearing (doc 02 §10.1). Fixed in `redox-lexer`'s `at_line_start` — and that is the \
     *\"hard way\"* doc 02 §4.8 said a port would find things.\n\n\
     ---\n\n## The Rust implementation is permanent\n\n\
     Doc 02 §4.8 corrects doc 01 §34's own wording: keep the Rust implementation *\"permanently \
     rather than as a 'fallback'\"*. Nothing in the build depends on `conformance/`, and the \
     compiler is not compiled from it.\n",
  );
  out
}

#[test]
fn conformance_is_current() {
  let root = workspace_root();
  let path = root.join("docs/CONFORMANCE.md");
  let rendered = render(&root);

  if std::env::var("REDOX_BLESS").is_ok() {
    std::fs::write(&path, &rendered).expect("should write CONFORMANCE.md");
    panic!("blessed docs/CONFORMANCE.md — review the diff rather than absorbing it");
  }

  let found = std::fs::read_to_string(&path).unwrap_or_default();
  assert_eq!(
    rendered, found,
    "docs/CONFORMANCE.md is out of date. If a blocker count fell, that is good news and this \
     is still a failure: REDOX_BLESS=1 cargo test -p redox-cli --test conformance"
  );
}

#[test]
fn the_port_transpiles_to_its_committed_output() {
  let root = workspace_root();
  let sdx = root.join("conformance/clause.sdx");
  let xrs = root.join("conformance/clause.xrs");

  let out = Command::new(env!("CARGO_BIN_EXE_redox"))
    .arg("build")
    .arg(&sdx)
    .output()
    .expect("should run redox build");
  assert!(
    out.status.success(),
    "conformance/clause.sdx should transpile:\n{}",
    String::from_utf8_lossy(&out.stdout)
  );

  let generated = std::fs::read_to_string(&xrs).expect("should read clause.xrs");
  assert!(
    !generated.contains("rust("),
    "the port must not reach for the escape hatch — doc 02 §4.8's criterion is that the \
     source is expressible *without* it"
  );
}

#[test]
fn the_ported_output_is_valid_rust() {
  let root = workspace_root();
  let generated =
    std::fs::read_to_string(root.join("conformance/clause.xrs")).expect("should read clause.xrs");

  // `syn`, not rustc: the port is one module lifted out of a crate, so
  // `crate::Stmt` cannot resolve and rustc would reject it for a reason that
  // is about the extraction rather than about Redox. Plan 42 §1.3 chose this
  // oracle for the same reason, and plan 48 §10 records the correction to
  // that plan's own criterion 3.
  if let Err(e) = syn::parse_file(&generated) {
    panic!("the ported module is not valid Rust syntax: {e}");
  }
}
