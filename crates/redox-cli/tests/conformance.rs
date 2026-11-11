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

mod support;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

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

/// A count as a word, because the heading it lands in reads as prose.
fn word(n: usize) -> String {
  const WORDS: &[&str] = &[
    "No", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven",
    "Twelve", "Thirteen", "Fourteen",
  ];
  WORDS
    .get(n)
    .map_or_else(|| n.to_string(), |w| (*w).to_string())
}

/// What the paragraph on permanence says about the crates that are *not*
/// ported.
///
/// **Two sentences, because the count reached zero.** The template read *"The
/// {rest} crates that are not ported are hand-written Rust"*, which plan 83
/// rendered as *"The no crates that are not ported"* — a generated document
/// going ungrammatical at exactly the milestone it exists to record. Doc 02
/// §4.8's rule is unchanged either way; what changes is where the Rust is.
fn remainder(unported: usize, rest: &str) -> String {
  if unported == 0 {
    return "**No crate's `src/` is hand-written Rust any more.** What stays Rust is what             plan 72 §6 named: the integration tests, which `AGENTS.md` requires to be             Rust asserting over Redox-generated code, and `crates/redox-driver/build.rs`,             which cargo runs on the host before anything exists."
      .to_string();
  }
  format!(
    "The {rest} crates that are not ported are hand-written Rust and stay that way until      a row says otherwise;"
  )
}

/// `a`, `b` and `c`. An English list, for the same reason.
fn and_list(items: &[String]) -> String {
  match items {
    [] => String::new(),
    [one] => one.clone(),
    [init @ .., last] => format!("{} and {last}", init.join(", ")),
  }
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
     REDOX_UPDATE=1 cargo test -p redox-cli --test conformance\n\
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

  // **Every number here is derived from `seed/PORTED`.** This section was
  // prose, and said "Two crates" and "110 files" with three crates ported —
  // because the list was in `port.rs` and the sentence was here. A generated
  // document that contradicts its generator reads as measured.
  let ported = support::ported();
  let names: Vec<String> = ported
    .iter()
    .map(|(c, plan)| format!("`{c}` (plan {plan})"))
    .collect();
  let ported_files: usize = per_crate
    .iter()
    .filter(|(n, _, _)| ported.iter().any(|(c, _)| c == n))
    .map(|(_, total, _)| *total)
    .sum();
  let n = ported.len();
  let unported = per_crate.len().saturating_sub(n);
  let rest = word(unported).to_lowercase();
  let _ = write!(
    out,
    "\n---\n\n## {heading} crates are no longer a count\n\n\
     The table above measures what *could* be ported. {heading} crates **have** been, and \
     for them the question is settled by rustc rather than by a blocker count: {list} are \
     written in Redox, their `.sdx` living in `crates/<crate>/sdx/` beside the `src/` it \
     generates, and the generated Rust is compiled as part of the workspace.\n\n\
     So **the compiler is now{partly} compiled from Redox** — {ported_files} files of it. \
     That corrects what this document said until plan 70: *\"nothing in the build depends \
     on `conformance/`, and the compiler is not compiled from it\"*, which was true when \
     plan 48 lifted one 45-line module out of a crate and checked it with `syn`, and \
     stopped being true the moment a whole crate's `src/` was generated.\n\n\
     Plan 48's probe is retired with that sentence. `conformance/clause.sdx` was \
     `clause.rs` expressed in Redox and checked by `syn::parse_file`, because one module \
     lifted out of a crate cannot resolve `crate::Stmt`; the real port has no extraction \
     to concede to, so the weaker oracle expired. Its no-escape-hatch assertion now runs \
     over all {ported_files} files in `port.rs`.\n\n\
     What it found still stands, and is the reason doc 02 §4.8 expected a port to be \
     worth doing: every field doc comment was silently discarded, and `redox fmt` deleted \
     them from the source, because the lexer classified a comment opening a continuation \
     line as *trailing* the previous line and then held it for a `Newline` a continuation \
     never produces. Plans 68 and 69 found three more of that class and two formatter \
     defects, and plan 73 four more in the toolchain that writes the port.\n\n\
     ---\n\n## The Rust implementation is permanent\n\n\
     Doc 02 §4.8 corrects doc 01 §34's own wording: keep the Rust implementation \
     *\"permanently rather than as a 'fallback'\"*. {remainder} The {lower} that \
     are ported keep their generated Rust **committed**, which is what doc 02 §4.8 means \
     by *\"commit the generated `.xrs` so the bootstrap never breaks\"* — the committed \
     output is the previous-version compiler every self-hosting language needs, stored as \
     reviewable source rather than as a binary.\n",
    heading = word(n),
    lower = word(n).to_lowercase(),
    list = and_list(&names),
    partly = if unported == 0 { "" } else { " partly" },
    remainder = remainder(unported, &rest),
  );
  out
}

#[test]
fn conformance_is_current() {
  let root = workspace_root();
  let path = root.join("docs/CONFORMANCE.md");
  let rendered = render(&root);

  if std::env::var("REDOX_UPDATE").is_ok() {
    std::fs::write(&path, &rendered).expect("should write CONFORMANCE.md");
    panic!("updated docs/CONFORMANCE.md — review the diff rather than absorbing it");
  }

  let found = std::fs::read_to_string(&path).unwrap_or_default();
  assert_eq!(
    rendered, found,
    "docs/CONFORMANCE.md is out of date. If a blocker count fell, that is good news and this \
     is still a failure: REDOX_UPDATE=1 cargo test -p redox-cli --test conformance"
  );
}
