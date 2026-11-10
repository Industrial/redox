//! The editor integrations, checked as strongly as each editor allows.
//!
//! Plan 49 (`docs/43 - 49 - Editor Integrations.md`). The row ships five
//! clients and the register asks for each to be *CI-checked*. Four of the five
//! editors are not installed on the machine this was developed on, so §1.4
//! makes the verdict a **ladder** rather than one word:
//!
//! | Client | Oracle | Strength |
//! |---|---|---|
//! | Neovim | `nvim --headless` loads the parser and runs the queries | the editor runs it |
//! | Cursor | `--install-extension` into a throwaway `--extensions-dir` | the editor installs it |
//! | VSCode | nothing — inherited from Cursor (§1.2) | a claim |
//! | Vim | nvim sourcing the legacy syntax file | a *compatible* editor runs it |
//! | Helix, Zed | the manifest parses and its keys are documented ones | well-formed |
//!
//! Each test's name carries its own rung, which is the point: `helix_config_parses`
//! cannot be misread as *Helix works*.
//!
//! # Every skip is reported
//!
//! A machine with no editors must pass this gate, and a check that quietly did
//! not run is indistinguishable from one that passed — plan 46 §1.3's argument.
//! So a missing tool prints a line naming what was skipped and why, and the
//! test returns.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn grammar_dir() -> PathBuf {
  root().join("tree-sitter-redox")
}

fn editors_dir() -> PathBuf {
  root().join("editors")
}

/// A scratch directory under the target dir, wiped on each run.
///
/// `temp_dir()` rather than the repository, for `examples.rs`'s reason: the
/// repository's own `rustfmt.toml` and tooling config apply inside the tree.
fn scratch(name: &str) -> PathBuf {
  let dir = std::env::temp_dir().join(format!("redox-editors-{name}"));
  let _ = std::fs::remove_dir_all(&dir);
  std::fs::create_dir_all(&dir).expect("should create a scratch directory");
  dir
}

/// Is a command runnable? Reported rather than assumed, since four of the five
/// editors are absent here.
fn available(program: &str) -> bool {
  Command::new(program)
    .arg("--version")
    .output()
    .is_ok_and(|out| out.status.success())
}

/// Print why a rung did not run. Stdout so `--nocapture` shows it; the test
/// still passes, which is deliberate (see the module docs).
fn skip(what: &str, why: &str) {
  println!("SKIP {what}: {why}");
}

// ---------------------------------------------------------------- the parser

/// Build plan 41's parser to a shared object.
///
/// Not committed: five platforms' binaries is a release problem, not an
/// editor-integration one (plan 49 §6), and each client's README gives this
/// same command.
fn build_parser(into: &Path) -> Option<PathBuf> {
  if !available("cc") {
    skip("parser build", "no `cc` on PATH");
    return None;
  }
  let so = into.join("redox.so");
  let src = grammar_dir().join("src");
  let out = Command::new("cc")
    .arg("-o")
    .arg(&so)
    .args(["-shared", "-fPIC", "-Os"])
    .arg(format!("-I{}", src.display()))
    .arg(src.join("parser.c"))
    .arg(src.join("scanner.c"))
    .output()
    .expect("cc should run");
  assert!(
    out.status.success(),
    "plan 41's parser should compile; every client's README gives this \
     command:\n{}",
    String::from_utf8_lossy(&out.stderr)
  );
  Some(so)
}

/// Run a Lua script under `nvim --headless -u NONE`, returning its output.
///
/// `-u NONE` so the result is about the committed config and not about whatever
/// the developer's `init.lua` happens to do.
fn nvim_lua(dir: &Path, script: &str) -> String {
  let path = dir.join("probe.lua");
  std::fs::write(&path, script).expect("should write the probe");
  let out = Command::new("nvim")
    .args(["--headless", "-u", "NONE", "-l"])
    .arg(&path)
    .output()
    .expect("nvim should run");
  format!(
    "{}{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  )
}

// ----------------------------------------------------------------- Neovim

/// Criteria 1 and 2 — the strongest rung: the editor runs the config.
///
/// Four claims in one nvim session, because starting nvim five times to make
/// five assertions would be the same session's work at five times the cost:
/// the parser loads from `editors/neovim/`'s own `setup`, every example parses
/// with no `ERROR` node, all three queries compile, and the highlight query
/// matches.
#[test]
fn neovim_runs_the_committed_config() {
  if !available("nvim") {
    skip("neovim", "no `nvim` on PATH");
    return;
  }
  let dir = scratch("neovim");
  let Some(parser) = build_parser(&dir) else {
    return;
  };

  let examples = sources(&root().join("examples"));
  assert!(
    examples.len() > 20,
    "expected the example corpus, found {}",
    examples.len()
  );

  // Placeholders rather than `format!`: Lua's table syntax is `{}`, and
  // escaping every brace in a sixty-line script to satisfy `format!` is how a
  // probe acquires a bug of its own.
  let script = NEOVIM_PROBE
    .replace(
      "@@RTP@@",
      &editors_dir().join("neovim").display().to_string(),
    )
    .replace("@@GRAMMAR@@", &grammar_dir().display().to_string())
    .replace("@@PARSER@@", &parser.display().to_string())
    .replace(
      "@@FILES@@",
      &examples
        .iter()
        .map(|p| format!("'{}'", p.display()))
        .collect::<Vec<_>>()
        .join(", "),
    );
  let out = nvim_lua(&dir, &script);

  assert!(
    out.contains("setup: ok"),
    "`require('redox').setup` should load from editors/neovim/:\n{out}"
  );
  assert!(
    out.contains("filetype: redox"),
    "and `.sdx` should be filetype `redox`:\n{out}"
  );
  assert!(
    out.contains("xrs filetype: rust"),
    "and `.xrs` should be filetype `rust` — it is ordinary Rust:\n{out}"
  );
  for name in ["highlights", "injections", "locals"] {
    assert!(
      out.contains(&format!("query {name}: ok")),
      "query {name} should compile under nvim, not only under the \
       tree-sitter CLI:\n{out}"
    );
  }
  assert!(
    out.contains("errors: 0"),
    "every example should parse with no ERROR node:\n{out}"
  );

  // Criterion 2: matching, not merely compiling. Plan 41's
  // `the_highlight_query_loads_and_matches` already asserts this through the
  // tree-sitter CLI; the value here is that **nvim's** query engine is a
  // different implementation, and it is the one an editor uses.
  let captures = out
    .lines()
    .find_map(|l| l.strip_prefix("captures: "))
    .and_then(|n| n.trim().parse::<usize>().ok())
    .unwrap_or_else(|| panic!("the probe should report a capture count:\n{out}"));
  assert!(
    captures > 50,
    "the highlight query should match the corpus, not merely compile; \
     nvim reported {captures} captures"
  );
  println!("neovim: {} examples, {captures} captures", examples.len());
}

const NEOVIM_PROBE: &str = r#"
vim.opt.runtimepath:prepend('@@RTP@@')
local ok, mod = pcall(require, 'redox')
if not ok then print('setup: FAILED ' .. tostring(mod)) return end
mod.setup({ grammar_dir = '@@GRAMMAR@@', parser = '@@PARSER@@', lsp = false })
print('setup: ok')

print('filetype: ' .. tostring(vim.filetype.match({ filename = 'a.sdx' })))
print('xrs filetype: ' .. tostring(vim.filetype.match({ filename = 'a.xrs' })))

for _, name in ipairs({ 'highlights', 'injections', 'locals' }) do
  local good, q = pcall(vim.treesitter.query.get, 'redox', name)
  if good and q ~= nil then
    print('query ' .. name .. ': ok')
  else
    print('query ' .. name .. ': FAILED ' .. tostring(q))
  end
end

local files = { @@FILES@@ }
local errors, captures = 0, 0
local hl = vim.treesitter.query.get('redox', 'highlights')
for _, file in ipairs(files) do
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, vim.fn.readfile(file))
  local parser = vim.treesitter.get_parser(buf, 'redox')
  local root = parser:parse()[1]:root()
  if root:has_error() then
    errors = errors + 1
    print('ERROR in ' .. file)
  end
  if hl then
    for _ in hl:iter_captures(root, buf, 0, -1) do captures = captures + 1 end
  end
  vim.api.nvim_buf_delete(buf, { force = true })
end
print('errors: ' .. errors)
print('captures: ' .. captures)
"#;

// -------------------------------------------------------------------- Vim

/// Criterion 4 — the third rung, and labelled one.
///
/// `nvim` runs legacy vimscript syntax files, so it can check this one. It is
/// **not vim**, and there is no `vim` here to be. The test's name says
/// `under_nvim` for that reason.
#[test]
fn vim_syntax_colours_a_keyword_under_nvim() {
  if !available("nvim") {
    skip("vim (via nvim)", "no `nvim` on PATH");
    return;
  }
  let dir = scratch("vim");
  let script = VIM_PROBE.replace("@@RTP@@", &editors_dir().join("vim").display().to_string());
  let out = nvim_lua(&dir, &script);

  assert!(
    out.contains("current_syntax: sdx"),
    "sourcing editors/vim/syntax/sdx.vim should set b:current_syntax:\n{out}"
  );
  assert!(
    out.contains("group: sdxKeyword"),
    "`data` at the start of a line should be sdxKeyword:\n{out}"
  );
  assert!(
    out.contains("comment group: sdxComment"),
    "and a `--` comment should be sdxComment, or keywords inside comments \
     would be coloured:\n{out}"
  );
  assert!(
    out.contains("in-comment group: sdxComment"),
    "including a keyword inside one — which is the whole reason the comment \
     rules come first in the file:\n{out}"
  );
}

const VIM_PROBE: &str = r#"
vim.opt.runtimepath:prepend('@@RTP@@')
vim.cmd('syntax on')
local buf = vim.api.nvim_create_buf(false, true)
vim.api.nvim_set_current_buf(buf)
vim.api.nvim_buf_set_lines(buf, 0, -1, false, {
  'data C = C { a : i32 }',
  '-- a comment mentioning data',
})
vim.bo.filetype = 'sdx'
vim.cmd('runtime! syntax/sdx.vim')
print('current_syntax: ' .. tostring(vim.b.current_syntax))
local function group(line, col)
  return vim.fn.synIDattr(vim.fn.synID(line, col, 1), 'name')
end
print('group: ' .. group(1, 1))
print('comment group: ' .. group(2, 1))
print('in-comment group: ' .. group(2, 26))
"#;

// --------------------------------------------------------- VSCode / Cursor

/// Criterion 3 — the second rung: the editor installs the extension.
///
/// A VSIX is an OPC zip, and `vsce` is absent and unfetchable here, so plan 49
/// §1.3 declined the packaging dependency. The implementation is the `zip`
/// crate rather than a hand-written archive writer, because AGENTS.md is
/// explicit that no plan implements a library function.
///
/// `REDOX_VSIX_OUT` keeps a copy, which is how `editors/vscode/README.md`
/// tells a reader to build one for a real install.
#[test]
fn the_vsix_installs_into_cursor() {
  let dir = scratch("vsix");
  let vsix = build_vsix(&dir.join("redox.vsix"));
  if let Ok(out) = std::env::var("REDOX_VSIX_OUT") {
    std::fs::copy(&vsix, &out).expect("should write REDOX_VSIX_OUT");
    println!("wrote {out}");
  }

  if !available("cursor") {
    skip(
      "cursor install",
      "no `cursor` on PATH — the VSIX was built and checked, not installed",
    );
    return;
  }

  // An explicit throwaway directory, so the developer's own extensions are
  // never touched. Criterion 8.
  let ext_dir = dir.join("extensions");
  std::fs::create_dir_all(&ext_dir).expect("should create an extensions dir");

  let install = Command::new("cursor")
    .arg("--extensions-dir")
    .arg(&ext_dir)
    .arg("--install-extension")
    .arg(&vsix)
    .arg("--force")
    .output()
    .expect("cursor should run");
  let said = format!(
    "{}{}",
    String::from_utf8_lossy(&install.stdout),
    String::from_utf8_lossy(&install.stderr)
  );
  assert!(
    install.status.success(),
    "cursor should install the VSIX. A hand-built manifest gets none of \
     `vsce`'s validation, so a mistake in it surfaces here:\n{said}"
  );

  let listed = Command::new("cursor")
    .arg("--extensions-dir")
    .arg(&ext_dir)
    .arg("--list-extensions")
    .output()
    .expect("cursor should run");
  let text = String::from_utf8_lossy(&listed.stdout);
  assert!(
    text.contains("redox-lang.redox"),
    "and should list it back as `redox-lang.redox`, which is the half of the \
     check that proves the install rather than the exit code:\n{text}"
  );
}

/// Write a VSIX from `editors/vscode/`.
///
/// The OPC layout: `[Content_Types].xml`, `extension.vsixmanifest`, and the
/// extension's own files under `extension/`.
fn build_vsix(path: &Path) -> PathBuf {
  let manifest: serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string(editors_dir().join("vscode/package.json"))
      .expect("should read the extension's package.json"),
  )
  .expect("package.json should be JSON");
  let publisher = manifest["publisher"].as_str().expect("a publisher");
  let name = manifest["name"].as_str().expect("a name");
  let version = manifest["version"].as_str().expect("a version");

  let file = std::fs::File::create(path).expect("should create the VSIX");
  let mut zip = zip::ZipWriter::new(file);
  // Stored rather than deflated. A VSIX does not need to be small, and the
  // compression features pull flate2, zlib-rs and a wasm-bindgen tree into the
  // dev graph to save kilobytes nobody transfers.
  let options: zip::write::FileOptions<()> =
    zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

  zip
    .start_file("[Content_Types].xml", options)
    .expect("should start a file");
  zip
    .write_all(CONTENT_TYPES.as_bytes())
    .expect("should write");

  zip
    .start_file("extension.vsixmanifest", options)
    .expect("should start a file");
  zip
    .write_all(
      VSIX_MANIFEST
        .replace("@@ID@@", name)
        .replace("@@PUBLISHER@@", publisher)
        .replace("@@VERSION@@", version)
        .as_bytes(),
    )
    .expect("should write");

  for relative in vsix_payload() {
    let from = editors_dir().join("vscode").join(&relative);
    zip
      .start_file(format!("extension/{relative}"), options)
      .expect("should start a file");
    zip
      .write_all(
        &std::fs::read(&from)
          .unwrap_or_else(|e| panic!("the extension should contain {}: {e}", from.display())),
      )
      .expect("should write");
  }
  zip.finish().expect("should finish the VSIX");
  path.to_path_buf()
}

/// Every file the extension needs, relative to `editors/vscode/`.
///
/// Listed rather than walked: `node_modules` lives in this directory after an
/// `npm install`, and a walk would put it in the VSIX. The install test is what
/// proves the list is complete.
fn vsix_payload() -> Vec<String> {
  [
    "package.json",
    "README.md",
    "language-configuration.json",
    "syntaxes/redox.tmLanguage.json",
    "src/extension.js",
  ]
  .into_iter()
  .map(str::to_string)
  .collect()
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension=".json" ContentType="application/json" />
  <Default Extension=".js" ContentType="application/javascript" />
  <Default Extension=".md" ContentType="text/markdown" />
  <Default Extension=".vsixmanifest" ContentType="text/xml" />
</Types>
"#;

const VSIX_MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011">
  <Metadata>
    <Identity Language="en-US" Id="@@ID@@" Version="@@VERSION@@" Publisher="@@PUBLISHER@@" />
    <DisplayName>Redox</DisplayName>
    <Description xml:space="preserve">Redox language support.</Description>
    <Tags>redox,rust</Tags>
    <Categories>Programming Languages</Categories>
    <GalleryFlags>Public</GalleryFlags>
  </Metadata>
  <Installation>
    <InstallationTarget Id="Microsoft.VisualStudio.Code" />
  </Installation>
  <Dependencies />
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Details" Path="extension/README.md" Addressable="true" />
  </Assets>
</PackageManifest>
"#;

// ------------------------------------------------------------- Helix, Zed

/// Criterion 5 for Helix — the weakest rung, and the name says `parses`.
///
/// There is no `hx` here, so the failure mode this file actually has is a typo
/// in a key Helix would **silently ignore**. A key allow-list taken from
/// Helix's documented schema is the strongest check available without the
/// editor; it is not Helix accepting the file.
#[test]
fn helix_config_parses_and_uses_documented_keys() {
  let text = std::fs::read_to_string(editors_dir().join("helix/languages.toml"))
    .expect("should read editors/helix/languages.toml");
  let parsed: toml::Table = text.parse().expect("languages.toml should be TOML");

  let languages = parsed["language"].as_array().expect("a [[language]] array");
  assert_eq!(languages.len(), 1, "one language, `redox`");
  let redox = languages[0].as_table().expect("a table");
  assert_eq!(redox["name"].as_str(), Some("redox"));
  assert_eq!(redox["file-types"].as_array().map(Vec::len), Some(1));

  const LANGUAGE_KEYS: &[&str] = &[
    "name",
    "scope",
    "injection-regex",
    "file-types",
    "comment-tokens",
    "block-comment-tokens",
    "indent",
    "language-servers",
    "auto-format",
    "roots",
    "shebangs",
    "text-width",
    "formatter",
    "grammar",
    "soft-wrap",
    "rulers",
  ];
  for key in redox.keys() {
    assert!(
      LANGUAGE_KEYS.contains(&key.as_str()),
      "`{key}` is not a key Helix documents for [[language]]; a key Helix \
       does not know is silently ignored, which is why this list exists"
    );
  }

  let grammars = parsed["grammar"].as_array().expect("a [[grammar]] array");
  assert_eq!(grammars.len(), 1);
  let grammar = grammars[0].as_table().expect("a table");
  assert_eq!(grammar["name"].as_str(), Some("redox"));
  let source = grammar["source"].as_table().expect("a source table");
  let relative = source["path"].as_str().expect("a path");
  assert!(
    editors_dir().join("helix").join(relative).is_dir(),
    "the grammar path should resolve from editors/helix/: {relative}"
  );

  let servers = parsed["language-server"]
    .as_table()
    .expect("a [language-server.*] table");
  assert!(
    servers.contains_key("redox-lsp"),
    "and the server the language references should be defined"
  );
  assert_eq!(
    servers["redox-lsp"].as_table().expect("a table")["command"].as_str(),
    Some("redox-lsp"),
    "plan 40's binary name"
  );
}

/// Criterion 5 for Zed, plus the query copies.
///
/// Zed reads its queries from inside the extension, so unlike Neovim — handed
/// the text at setup time — a second copy is unavoidable. A second copy of a
/// file is a file that drifts, so the gate compares them byte for byte, which
/// is the same mechanism the golden corpus uses.
#[test]
fn zed_config_parses_and_its_queries_match_the_grammar() {
  let zed = editors_dir().join("zed");
  let manifest: toml::Table = std::fs::read_to_string(zed.join("extension.toml"))
    .expect("should read editors/zed/extension.toml")
    .parse()
    .expect("extension.toml should be TOML");

  const MANIFEST_KEYS: &[&str] = &[
    "id",
    "name",
    "description",
    "version",
    "schema_version",
    "authors",
    "repository",
    "grammars",
    "language_servers",
    "lib",
    "snippets",
    "themes",
    "icon_themes",
    "slash_commands",
    "context_servers",
  ];
  for key in manifest.keys() {
    assert!(
      MANIFEST_KEYS.contains(&key.as_str()),
      "`{key}` is not a key Zed documents for extension.toml"
    );
  }
  assert_eq!(manifest["id"].as_str(), Some("redox"));
  assert_eq!(manifest["schema_version"].as_integer(), Some(1));

  let grammar = manifest["grammars"].as_table().expect("a grammars table")["redox"]
    .as_table()
    .expect("a redox grammar");
  let relative = grammar["path"].as_str().expect("a path");
  assert!(
    zed.join(relative).is_dir(),
    "the grammar path should resolve from editors/zed/: {relative}"
  );

  let config: toml::Table = std::fs::read_to_string(zed.join("languages/redox/config.toml"))
    .expect("should read the language config")
    .parse()
    .expect("config.toml should be TOML");
  assert_eq!(config["name"].as_str(), Some("Redox"));
  assert_eq!(config["grammar"].as_str(), Some("redox"));
  assert_eq!(
    config["path_suffixes"].as_array().map(Vec::len),
    Some(1),
    "`sdx` only — `.xrs` is ordinary Rust and gets Zed's own Rust support"
  );

  for name in ["highlights", "injections", "locals"] {
    let committed = zed.join(format!("languages/redox/{name}.scm"));
    let source = grammar_dir().join(format!("queries/{name}.scm"));
    assert_eq!(
      std::fs::read_to_string(&committed)
        .unwrap_or_else(|e| panic!("should read {}: {e}", committed.display())),
      std::fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("should read {}: {e}", source.display())),
      "editors/zed/languages/redox/{name}.scm has drifted from the grammar's \
       copy. Re-copy with `cp tree-sitter-redox/queries/*.scm \
       editors/zed/languages/redox/`"
    );
  }
}

// ------------------------------------------------- one keyword list, three

/// Every keyword the grammar defines is coloured by all three highlighters.
///
/// Three highlighters carry a keyword list — plan 41's `highlights.scm`, this
/// row's Vim syntax file and its TextMate grammar — and `grammar.js` is the
/// fourth list, the only one the compiler's own gate already exercises. So the
/// grammar is the source and the other three are checked against it.
///
/// It found the gap it was written for. Plan 45 added `rust`, `unsafe`,
/// `extern` and `union` to the grammar and to the compiler and **no
/// highlighter coloured any of them**, because nothing compared the lists.
///
/// # The floor is a non-vacuity guard, not a count
///
/// It read `> 40` until plan 55 leaf 7 retired `then` and moved `do` out of
/// the derived set (see [`NOT_KEYWORDS`](grammar_keywords)), leaving 39 — and
/// a floor that tracks the real figure fails on every retirement while
/// catching nothing a lower one misses. What it is for is a scan that has
/// silently stopped working, and the six named members below are the stronger
/// half of that: `MINIMUM_ROWS` in `construct_coverage.rs` records the same
/// reasoning, having been wrong twice.
#[test]
fn every_grammar_keyword_is_coloured() {
  let keywords = grammar_keywords();
  assert!(
    keywords.len() > 30,
    "expected the grammar's keyword set, found {}: {keywords:?}",
    keywords.len()
  );
  for expected in ["rust", "unsafe", "extern", "union", "deriving", "otherwise"] {
    assert!(
      keywords.contains(&expected.to_string()),
      "`{expected}` should be in the derived set, or this test is not reading \
       the grammar it thinks it is"
    );
  }

  let highlights = std::fs::read_to_string(grammar_dir().join("queries/highlights.scm"))
    .expect("should read highlights.scm");
  let keyword_block = between(&highlights, "[", "] @keyword")
    .expect("highlights.scm should have a bracketed @keyword alternation");
  let vim = std::fs::read_to_string(editors_dir().join("vim/syntax/sdx.vim"))
    .expect("should read the Vim syntax file");
  let vim_keywords: String = vim
    .lines()
    .filter(|l| l.trim_start().starts_with("syn keyword sdxKeyword"))
    .collect::<Vec<_>>()
    .join(" ");
  let textmate =
    std::fs::read_to_string(editors_dir().join("vscode/syntaxes/redox.tmLanguage.json"))
      .expect("should read the TextMate grammar");

  let mut missing = Vec::new();
  for keyword in &keywords {
    if !keyword_block.contains(&format!("\"{keyword}\"")) {
      missing.push(format!("  {keyword}: tree-sitter queries/highlights.scm"));
    }
    if !vim_keywords.split_whitespace().any(|w| w == keyword) {
      missing.push(format!("  {keyword}: editors/vim/syntax/sdx.vim"));
    }
    if !textmate.contains(&format!("|{keyword}|"))
      && !textmate.contains(&format!("({keyword}|"))
      && !textmate.contains(&format!("|{keyword})"))
    {
      missing.push(format!(
        "  {keyword}: editors/vscode/syntaxes/redox.tmLanguage.json"
      ));
    }
  }
  assert!(
    missing.is_empty(),
    "the grammar defines keywords that a highlighter does not colour. Three \
     highlighters that disagree are worse than one:\n{}",
    missing.join("\n")
  );
}

/// And every keyword a highlighter colours is one the grammar still defines.
///
/// [`every_grammar_keyword_is_coloured`] runs grammar → highlighters. This is
/// the other direction, and it is the one that has failed twice.
///
/// A retirement leaves a dead word behind. Plan 49's cross-check **found four**
/// when it was written; plan 55 leaf 7 retired `then` and found four more — in
/// `highlights.scm`, the Zed copy of it, `sdx.vim` and `redox.tmLanguage.json`
/// — and found them **by grep**, because nothing asserted this direction. A
/// highlighter that colours a word the compiler rejects tells an author the
/// word is live, which is the opposite of what a highlighter is for.
///
/// # Why only two of the three highlighters
///
/// `queries/highlights.scm` needs no check here: a query naming a node type
/// the grammar does not have is a **hard error** from `tree-sitter`, not a
/// dead pattern, and `tree_sitter.rs` runs both `query` and — since plan 65 —
/// `test`. That is how leaf 7 learned `"then"` and then `"do"` had to go from
/// it. The Vim and TextMate lists are plain text that nothing validates, so
/// they are what this covers.
///
/// # The oracle is the lexer, not `grammar.js`
///
/// [`grammar_keywords`] is the right source for the *forward* direction, which
/// asks whether the grammar's own keywords are coloured. It is the wrong one
/// here, and the first version of this test used it and produced two false
/// positives — `type` and `guard`, both live.
///
/// It is a **text scan for quoted literals**, so it misses any word the grammar
/// spells another way. Probed: `type Pair = (u64, u64)` parses with **zero**
/// `ERROR` nodes under `tree-sitter parse` while `'type'` appears nowhere in
/// `grammar.js`, because a type alias is reached through a rule rather than a
/// literal. A scan that under-collects is harmless when it drives a *demand*
/// and wrong when it drives a *refusal*.
///
/// So this asks `Keyword::from_text` — the compiler's own keyword table, and
/// the thing an author's editor is supposed to agree with. A word it rejects is
/// a word `redox build` will not accept, which is exactly the claim a
/// highlighter makes by colouring it.
#[test]
fn every_coloured_keyword_is_still_in_the_grammar() {
  /// Coloured deliberately, and not a `Keyword`.
  ///
  /// `export` is a **visibility** rather than a keyword — a regex in
  /// `grammar.js`, matched by `(visibility)`, and `Keyword::from_text` does not
  /// know it. Colouring it is right: an author types it where a Rust author
  /// types `pub`.
  ///
  /// `Self` is a type rather than a keyword, coloured by its own
  /// `variable.language.self` rule; it reaches a list this test reads only in
  /// Vim, under the separate `sdxSelf` group.
  ///
  /// `from` is the interesting one: it is Redox's **only contextual keyword**.
  /// `error_variant.rs` reads it as `TokenKind::Ident(text) if text == "from"`,
  /// so the lexer hands it over as an identifier and position is what makes it
  /// syntax — `examples/errors.sdx` has `from std::num::ParseIntError as
  /// Parse`. It is in no `Keyword` table and in no `grammar.js` literal, which
  /// is why neither direction of this cross-check finds it on its own.
  ///
  /// Colouring it is right, and it carries one known cost: because it is an
  /// identifier, `from = 1` is a legal binding and both highlighters will
  /// colour that `from` as syntax. That is a mis-colouring the compiler cannot
  /// be asked to prevent, and the alternative — leaving `errors` blocks
  /// uncoloured — is worse.
  const COLOURED_BUT_NOT_A_KEYWORD: &[&str] = &["export", "Self", "from"];

  // The oracle's own floor: a `from_text` that stopped matching would
  // otherwise make every word below look dead.
  for live in ["if", "else", "case", "guard", "type", "do"] {
    assert!(
      redox_lexer::Keyword::from_text(live).is_some(),
      "`{live}` should still be a Keyword, or this test is not reading the lexer it thinks \
       it is"
    );
  }
  assert!(
    redox_lexer::Keyword::from_text("then").is_none(),
    "`then` was retired at plan 55 leaf 7 and should not be a Keyword — if it is back, this \
     test's premise has changed"
  );

  let known = |word: &str| {
    redox_lexer::Keyword::from_text(word).is_some() || COLOURED_BUT_NOT_A_KEYWORD.contains(&word)
  };

  let mut dead = Vec::new();

  // Vim: the words on `syn keyword sdxKeyword` lines. The other groups —
  // `sdxBoolean`, `sdxSelf` — colour constructs that are not keywords.
  let vim = std::fs::read_to_string(editors_dir().join("vim/syntax/sdx.vim"))
    .expect("should read the Vim syntax file");
  for line in vim
    .lines()
    .filter(|l| l.trim_start().starts_with("syn keyword sdxKeyword"))
  {
    for word in line.split_whitespace().skip(3) {
      if !known(word) {
        dead.push(format!("  {word}: editors/vim/syntax/sdx.vim"));
      }
    }
  }

  // TextMate: the alternation of the `keyword.control.redox` pattern.
  let textmate =
    std::fs::read_to_string(editors_dir().join("vscode/syntaxes/redox.tmLanguage.json"))
      .expect("should read the TextMate grammar");
  let control = textmate
    .split("\"keyword.control.redox\"")
    .nth(1)
    .and_then(|rest| between(rest, "\\\\b(", ")\\\\b"))
    .expect(
      "the TextMate grammar should carry a `keyword.control.redox` pattern whose match is a \
       `\\b(a|b|…)\\b` alternation — if that shape changed, this check needs rewriting rather \
       than deleting",
    );
  let alternation: Vec<&str> = control.split('|').collect();
  assert!(
    alternation.len() > 30,
    "parsed only {} words out of the TextMate alternation, so this check is not reading what \
     it thinks it is: {control}",
    alternation.len()
  );
  for word in alternation {
    if !known(word) {
      dead.push(format!(
        "  {word}: editors/vscode/syntaxes/redox.tmLanguage.json"
      ));
    }
  }

  assert!(
    dead.is_empty(),
    "a highlighter colours {} word(s) the lexer does not accept as a keyword:\n{}\n\nA dead \
     keyword tells an author the word is live, which is the opposite of what a highlighter is \
     for. Either remove it, or — if it is deliberately coloured while not being a `Keyword`, \
     as `export` and `Self` are — add it to COLOURED_BUT_NOT_A_KEYWORD with the reason.",
    dead.len(),
    dead.join("\n"),
  );
}

/// And every **operator** a highlighter colours lexes as one token.
///
/// [`every_coloured_keyword_is_still_in_the_grammar`] closes the reverse
/// direction for keywords. It does not cover operators, and plan 66 found what
/// that costs: `editors/vim/syntax/sdx.vim` and
/// `editors/vscode/syntaxes/redox.tmLanguage.json` were both still colouring
/// **`?.`**, which plan 57 deleted, and **`<|`**, which has never been a Redox
/// token at all. Two dead spellings in the same regex that row was editing,
/// found by eye because no gate asked.
///
/// # The oracle is the lexer's behaviour, not a table
///
/// `OPERATOR_TABLE` is private to `redox-lexer`, so an integration test cannot
/// read it. That turns out to be the better position: this asks
/// [`redox_lexer::lex`] whether the spelling **is one token**, which is the
/// property a highlighter is claiming by colouring it as a unit.
///
/// A dead spelling fails that for free. `?.` lexes as `Question` then `Dot` —
/// two tokens, since plan 57 retired `QuestionDot` — and `<|` as `Lt` then
/// `Or`. Neither needs a list to be recognised as gone.
#[test]
fn every_coloured_operator_lexes_as_one_token() {
  let vim = std::fs::read_to_string(editors_dir().join("vim/syntax/sdx.vim"))
    .expect("should read the Vim syntax file");
  let textmate =
    std::fs::read_to_string(editors_dir().join("vscode/syntaxes/redox.tmLanguage.json"))
      .expect("should read the TextMate grammar");

  let mut sources: Vec<(&str, Vec<String>)> = Vec::new();

  // Vim: the `sdxOperator` match, whose alternatives are separated by `\|`.
  let line = vim
    .lines()
    .find(|l| l.trim_start().starts_with("syn match   sdxOperator"))
    .expect("the Vim syntax file should carry an `sdxOperator` match");
  // Not `between`: it locates the *close* marker first, so it cannot handle a
  // pair of identical delimiters.
  let body = line.split('"').nth(1).expect("the match should be quoted");
  sources.push((
    "editors/vim/syntax/sdx.vim",
    body.split("\\|").map(|a| a.replace('\\', "")).collect(),
  ));

  // TextMate: the `keyword.operator.redox` match, alternatives on `|`.
  let control = textmate
    .split("\"keyword.operator.redox\"")
    .nth(1)
    .and_then(|rest| rest.split("\"match\": \"").nth(1))
    .and_then(|rest| rest.split('"').next())
    .expect("the TextMate grammar should carry a `keyword.operator.redox` match");
  sources.push((
    "editors/vscode/syntaxes/redox.tmLanguage.json",
    textmate_alternatives(control),
  ));

  let mut dead = Vec::new();
  let mut checked = 0;

  for (file, alternatives) in &sources {
    for alt in alternatives {
      // A character class or a lookahead is not a spelling. `[` and `(` never
      // begin a Redox operator, so this skips exactly the regex machinery.
      if alt.is_empty() || alt.starts_with('[') || alt.contains('(') {
        continue;
      }
      let Ok(tokens) = redox_lexer::lex(alt) else {
        dead.push(format!("  {alt}: {file} — does not lex at all"));
        continue;
      };
      // `is_structural` is `pub(crate)`, so the layout kinds and the terminator
      // are named here rather than asked for.
      let real: Vec<_> = tokens
        .iter()
        .map(|t| &t.kind)
        .filter(|k| {
          !matches!(
            k,
            redox_lexer::TokenKind::Newline
              | redox_lexer::TokenKind::Indent
              | redox_lexer::TokenKind::Dedent
              | redox_lexer::TokenKind::Eof
          )
        })
        .collect();
      checked += 1;
      if real.len() != 1 {
        dead.push(format!(
          "  {alt}: {file} — lexes as {} tokens, {real:?}",
          real.len()
        ));
      }
    }
  }

  // Twenty today, ten from each file, after the character classes and the
  // lambda's doubled backslash are skipped. A floor rather than an equality,
  // so adding an operator does not have to touch this number — but low enough
  // that an extraction which silently stopped working fails here.
  assert!(
    checked >= 16,
    "only {checked} operator spellings were extracted, so this test is not reading the \
     regexes it thinks it is"
  );

  assert!(
    dead.is_empty(),
    "a highlighter colours {} operator spelling(s) that are not one token:\n{}\n\nA \
     highlighter that colours `?.` after it was retired tells an author the spelling is \
     live. Either remove the alternative, or — if it is deliberately coloured as two \
     tokens — move it out of the operator match so this check does not see it.",
    dead.len(),
    dead.join("\n"),
  );
}

/// Split a TextMate pattern on its **alternation** pipes, and nothing else.
///
/// Needed because the two highlighters spell alternation differently: Vim uses
/// `\|`, so a bare `|` there is literal and splitting on `\|` leaves a
/// character class intact. TextMate is the reverse — alternation is a bare `|`
/// and a literal pipe is `\|` — so a naive split tears
/// `[-+*/%<>=!&|^~]` in half and reports `^~]` as a dead operator. It did,
/// before this existed.
///
/// So: skip an escaped character, and ignore a `|` inside `[…]`. The returned
/// alternatives have their escapes stripped, which is what makes them
/// lexable as source text.
///
/// # Two escape layers, undone in order
///
/// The file is JSON *containing* a regex, so `\\\\` on disk is the regex `\\`
/// which is one literal backslash, and `\\?` on disk is the regex `\?` which
/// is one literal `?`. Undoing them in one pass eats the alternation: the
/// lambda's `\\\\|\\?\\?` reads as *escaped-backslash, escaped-pipe* and
/// yields the alternative `\|??`, which was reported as a dead operator before
/// this was split in two.
fn textmate_alternatives(pattern: &str) -> Vec<String> {
  // Layer one: JSON. After this, a single `\` is the regex's own escape.
  let pattern = pattern.replace("\\\\", "\\");

  let mut out = Vec::new();
  let mut current = String::new();
  let mut in_class = false;
  let mut chars = pattern.chars().peekable();

  while let Some(c) = chars.next() {
    match c {
      // Layer two: the regex. `\X` is a literal X, whatever X is.
      '\\' => {
        if let Some(escaped) = chars.next() {
          current.push(escaped);
        }
      }
      '[' => {
        in_class = true;
        current.push(c);
      }
      ']' => {
        in_class = false;
        current.push(c);
      }
      '|' if !in_class => {
        out.push(std::mem::take(&mut current));
      }
      _ => current.push(c),
    }
  }
  out.push(current);
  out
}

/// The grammar's keyword literals: bare lowercase strings in `grammar.js` that
/// are not `field()` names.
///
/// Deliberately a text scan and not a JS parser — the alternative is a
/// JavaScript runtime in the gate. It over-collects, so the exclusions below
/// are named individually rather than filtered by a pattern, and the test
/// above asserts six known members so a scan that silently stopped working
/// fails instead of passing vacuously.
fn grammar_keywords() -> Vec<String> {
  /// Has its own capture in `highlights.scm`, so it is not a `@keyword`:
  /// `true`/`false` are `(boolean)`, `self` is `(self_expression)`, `do` is
  /// `(reserved_word)`, and `redox` is the grammar's own name. `export` is a
  /// *regex* in `grammar.js` rather than a literal — matched by
  /// `(visibility)` — so it is not in the derived set at all.
  ///
  /// `do` joined the list at plan 55 leaf 7, which retired its construct and
  /// kept the word reserved. `grammar.js` declares it through `reserved` and a
  /// `reserved_word` rule rather than as a literal in a production, so the
  /// node the query can name is `(reserved_word)` — and this scan reads
  /// literals, so it finds `"do"` and would ask for a `"do"` node that does
  /// not exist.
  const NOT_KEYWORDS: &[&str] = &["true", "false", "self", "redox", "do"];

  let text =
    std::fs::read_to_string(grammar_dir().join("grammar.js")).expect("should read the grammar");
  let code: String = text
    .lines()
    .filter(|l| !l.trim_start().starts_with("//"))
    .collect::<Vec<_>>()
    .join("\n");

  let mut found: Vec<String> = Vec::new();
  for literal in bare_lowercase_literals(&code) {
    if NOT_KEYWORDS.contains(&literal.as_str()) {
      continue;
    }
    // A `field('name', …)` is a field, not a keyword.
    if code.contains(&format!("field('{literal}'"))
      || code.contains(&format!("field(\"{literal}\""))
    {
      continue;
    }
    if !found.contains(&literal) {
      found.push(literal);
    }
  }
  found.sort();
  found
}

/// Double-quoted or single-quoted runs of `[a-z_]`, two characters or more.
fn bare_lowercase_literals(code: &str) -> Vec<String> {
  let mut found = Vec::new();
  let bytes: Vec<char> = code.chars().collect();
  let mut i = 0;
  while i < bytes.len() {
    let quote = bytes[i];
    if quote != '"' && quote != '\'' {
      i += 1;
      continue;
    }
    let mut j = i + 1;
    while j < bytes.len() && bytes[j] != quote && bytes[j] != '\n' {
      j += 1;
    }
    if j < bytes.len() && bytes[j] == quote {
      let inner: String = bytes[i + 1..j].iter().collect();
      if inner.len() >= 2 && inner.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        found.push(inner);
      }
      i = j + 1;
    } else {
      i += 1;
    }
  }
  found
}

/// The text between the first `open` and the first following `close`.
fn between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
  let end = text.find(close)?;
  let start = text[..end].rfind(open)?;
  Some(&text[start + open.len()..end])
}

// ---------------------------------------------------------- paths and docs

/// Criterion 6: no config names a path that does not exist.
///
/// A config pointing at a moved file is the failure mode these clients
/// actually have, and four of the five editors are not here to report it.
#[test]
fn every_path_a_config_names_exists() {
  let vscode = editors_dir().join("vscode");
  let manifest: serde_json::Value =
    serde_json::from_str(&std::fs::read_to_string(vscode.join("package.json")).expect("read"))
      .expect("package.json should be JSON");
  for relative in [
    manifest["main"].as_str().expect("a main"),
    manifest["contributes"]["languages"][0]["configuration"]
      .as_str()
      .expect("a language configuration"),
    manifest["contributes"]["grammars"][0]["path"]
      .as_str()
      .expect("a grammar path"),
  ] {
    let path = vscode.join(relative.trim_start_matches("./"));
    assert!(
      path.is_file(),
      "editors/vscode/package.json names {relative}, which is not there"
    );
  }

  // The VSIX payload list, which a walk would get wrong once `node_modules`
  // exists.
  for relative in vsix_payload() {
    assert!(
      vscode.join(&relative).is_file(),
      "the VSIX payload names {relative}, which is not there"
    );
  }

  for relative in [
    "neovim/lua/redox/init.lua",
    "vim/syntax/sdx.vim",
    "vim/ftplugin/sdx.vim",
    "vim/ftdetect/sdx.vim",
    "helix/languages.toml",
    "zed/extension.toml",
    "zed/languages/redox/config.toml",
  ] {
    assert!(
      editors_dir().join(relative).is_file(),
      "editors/{relative} should exist"
    );
  }

  // Plan 40's binary, named by three of the five configs.
  assert!(
    root().join("crates/redox-lsp/src/main.rs").is_file(),
    "the configs name `redox-lsp`, so this workspace should build one"
  );
}

/// Criterion 6's other half: every file a client needs is **tracked**.
///
/// `every_path_a_config_names_exists` asks the filesystem, and that is not the
/// same question. `.gitignore` carries a bare `config.toml` from the initial
/// template, which matches at any depth — so Zed's `languages/redox/config.toml`
/// was written, asserted present, and **silently untracked**: a fresh clone
/// would not have it and the path sweep would still pass, because the developer's
/// working tree does.
///
/// Caught by `git status` refusing to stage it, which is luck. This asks git.
#[test]
fn every_path_a_config_names_is_tracked() {
  let out = Command::new("git")
    .current_dir(root())
    .args(["ls-files", "--", "editors"])
    .output()
    .expect("git should run");
  assert!(out.status.success(), "`git ls-files` should succeed");
  let tracked: Vec<String> = String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(str::to_string)
    .collect();
  assert!(
    tracked.len() > 15,
    "expected editors/ to be tracked, found {} files",
    tracked.len()
  );

  let mut untracked = Vec::new();
  for relative in vsix_payload()
    .into_iter()
    .map(|p| format!("vscode/{p}"))
    .chain(
      [
        "README.md",
        "neovim/README.md",
        "neovim/lua/redox/init.lua",
        "vim/README.md",
        "vim/syntax/sdx.vim",
        "vim/ftplugin/sdx.vim",
        "vim/ftdetect/sdx.vim",
        "helix/README.md",
        "helix/languages.toml",
        "zed/README.md",
        "zed/extension.toml",
        "zed/languages/redox/config.toml",
        "zed/languages/redox/highlights.scm",
        "zed/languages/redox/injections.scm",
        "zed/languages/redox/locals.scm",
      ]
      .into_iter()
      .map(str::to_string),
    )
  {
    if !tracked.contains(&format!("editors/{relative}")) {
      untracked.push(format!("  editors/{relative}"));
    }
  }
  assert!(
    untracked.is_empty(),
    "these are on disk but not tracked, so a fresh clone would not have \
     them. Check `.gitignore` — a bare pattern there matches at any \
     depth:\n{}",
    untracked.join("\n")
  );
}

/// Criterion 7: install steps live beside each client, and the index links
/// all five.
#[test]
fn every_client_documents_its_own_install() {
  let index = std::fs::read_to_string(editors_dir().join("README.md"))
    .expect("editors/README.md should exist");
  for client in ["neovim", "vim", "vscode", "helix", "zed"] {
    let readme = editors_dir().join(client).join("README.md");
    let text = std::fs::read_to_string(&readme)
      .unwrap_or_else(|e| panic!("editors/{client}/README.md should exist: {e}"));
    assert!(
      text.contains("## Install"),
      "editors/{client}/README.md should hold its own install steps — the \
       register asks for them beside each client, not buried in one README"
    );
    assert!(
      text.contains("## Verified by"),
      "and should say which rung of plan 49 §1.4's ladder it is on, so a \
       green gate is not read as more than it is"
    );
    assert!(
      index.contains(&format!("{client}/README.md")),
      "editors/README.md should link editors/{client}/README.md"
    );
  }
}

/// Every `.sdx` in a directory, sorted.
fn sources(dir: &Path) -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
    .unwrap_or_else(|e| panic!("should read {}: {e}", dir.display()))
    .map(|entry| entry.expect("should read an entry").path())
    .filter(|path| path.extension().is_some_and(|x| x == "sdx"))
    .collect();
  found.sort();
  found
}
