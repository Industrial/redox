//! The seed's whole behaviour, as one function returning an exit code.

mod arguments;
mod report;
mod tidy;
mod walk;

use arguments::arguments;
use redox_diag::render;
use redox_driver::Silent;
use redox_driver::compile_dir_with;
use report::report;
use tidy::tidy;
use walk::walk;

/// Parse the arguments, transpile the tree, and answer with an exit code.
///
/// An exit code rather than a `Result`, because `main` has nothing to add to
/// it and a seed has no error type worth exporting. The three codes match
/// `redox-cli`'s so a caller cannot tell them apart on the outcome.
pub(crate) fn run() -> i32 {
  let Some((src, out)) = arguments() else {
    eprintln!("usage: redox-seed <src-dir> <out-dir>");
    return 2;
  };

  // Every directory, not just the root: `compile_dir` does not descend, and
  // `walk`'s doc records why that is its own row's problem rather than this
  // one's.
  let mut diagnostics = Vec::new();
  let mut failed = false;
  for (from, into) in walk(&src, &out) {
    match compile_dir_with(&from, &into, &Silent) {
      Ok(compiled) => report(&compiled),
      // **Collected, not returned.** A seed reporting the first directory's
      // errors and stopping would make a clean clone take as many runs to
      // diagnose as it has broken directories. Everything that compiles is
      // still written, which is what `init.sh` wants: the failure is loud and
      // the tree is as complete as it can be.
      Err(found) => {
        failed = true;
        diagnostics.extend(found);
      }
    }
    // The cache `compile_dir_with` just wrote into the destination. Right for
    // plan 09's `OUT_DIR`, wrong for a crate's `src/` — `tidy`'s doc has the
    // three reasons.
    tidy(&into);
  }

  if failed {
    // `plain: true`, and not because stderr might not be a terminal. The
    // graphical renderer is what reaches for `miette`, and the seed's entire
    // reason for existing is to be the smallest binary that transpiles — see
    // this crate's module doc.
    eprintln!("{}", render(&diagnostics, true));
    return 1;
  }
  0
}
