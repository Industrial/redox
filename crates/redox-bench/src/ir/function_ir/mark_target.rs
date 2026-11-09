/// The source with the compared function marked so rustc must emit it.
///
/// # Why this is necessary, and what it replaced
///
/// An rlib at `-O` does **not** codegen a `pub fn` at all: the function is
/// stored as MIR for a downstream crate to inline, and `--emit=llvm-ir`
/// produces a module with no `define` for it. Zero definitions, for both
/// halves — and two empty bodies compare equal, which is a vacuous pass. The
/// test's non-empty assertion is what caught it, and is why that assertion
/// exists.
///
/// `#[no_mangle]` forces the symbol to be emitted and exported.
/// `#[inline(never)]` keeps it whole. Together they make the function
/// something rustc has to produce.
///
/// `no_mangle` also makes the comparison **simpler** than plan 44 decision 5
/// planned: the symbol becomes literally the function's name, with no crate
/// name or path hash in it, so holding `--crate-name` constant stops being
/// what makes the two sides comparable. The flag stays anyway — it costs
/// nothing and it keeps the reasoning true if the marking ever changes.
///
/// `-Ccodegen-units=1` stays for a different reason, which the flag's own
/// comment records: with several units rustc writes one `.ll` per unit and
/// `-o` names only one of them.
pub(crate) fn mark_target(source: &str, name: &str) -> String {
  let signature = format!("fn {name}(");
  source
    .lines()
    .map(|line| {
      let trimmed = line.trim_start();
      if trimmed.starts_with(&signature) || trimmed.starts_with(&format!("pub {signature}")) {
        let body = if trimmed.starts_with("pub ") {
          line.to_string()
        } else {
          format!("pub {line}")
        };
        format!("#[no_mangle]\n#[inline(never)]\n{body}")
      } else {
        line.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}
