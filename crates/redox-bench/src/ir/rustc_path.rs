/// The rustc this row compiles with.
///
/// `REDOX_RUSTC` is baked in by `redox-cli`'s build script, which AGENTS.md
/// records as the only trustworthy way to find rustc here: **neither obvious
/// source of a path is reliable**, because `PATH` finds NixOS's `stub-ld` shim
/// under `blaze test` and `RUSTC` finds blaze's own shim under `blaze build`.
/// That script does not trust a candidate — it runs `--version` and checks the
/// banner.
///
/// This crate cannot read `redox-cli`'s baked value, so it has a build script
/// of its own that bakes `REDOX_RUSTC` the same way — and that script is what
/// makes the first candidate below resolve. Probing at run time alone is not
/// enough: under `blaze test` every candidate visible here is a shim, so all
/// four of this crate's tests failed with `no rustc answered --version like
/// rustc` while passing under `cargo nextest`. A build script sees the real
/// toolchain on `PATH`; a test binary does not.
///
/// The run-time probe stays regardless, because verifying rather than
/// assuming is the rule and the baked value is still only a candidate.
pub fn rustc_path() -> Option<String> {
  let candidates = [
    // `env!`, not `std::env::var`. `cargo::rustc-env` from a build script
    // bakes the value in at COMPILE time; it is not in the process
    // environment at run time. Reading it with `std::env::var` therefore
    // always missed, which left `PATH`'s stub as the only candidate under
    // `blaze test` and failed all four tests — while passing under cargo,
    // where `PATH` happens to hold a real rustc and hid the mistake.
    Some(env!("REDOX_RUSTC").to_string()),
    std::env::var("REDOX_RUSTC").ok(),
    std::env::var("RUSTC").ok(),
    Some("rustc".to_string()),
  ];
  for candidate in candidates.into_iter().flatten() {
    let Ok(output) = std::process::Command::new(&candidate)
      .arg("--version")
      .output()
    else {
      continue;
    };
    if !output.status.success() {
      continue;
    }
    // A path that exists is not a path that works. The banner is the check.
    if String::from_utf8_lossy(&output.stdout).starts_with("rustc ") {
      return Some(candidate);
    }
  }
  None
}
