//! The VSCode/Cursor rungs of the ladder, and the VSIX packager behind them.
//!
//! Split out of `editors.rs` by plan 71 leaf 2, for the reason AGENTS.md gives:
//! no source file over 1,500 lines, enforced by `line_cap.rs`. Leaf 2 added a
//! `node_modules` walk, a derived `[Content_Types].xml` and a second Cursor
//! rung, and that took the parent to 1,513 — so the whole VSCode/Cursor
//! section lives here and `editors.rs` keeps the other four clients.
//!
//! The helpers stay the parent's (`root`, `editors_dir`, `scratch`,
//! `available`, `skip`): they are shared by all five clients, and a copy per
//! module is how a ladder starts reporting different strengths for the same
//! rung.

use super::{available, editors_dir, scratch, skip};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Criterion 1 of plan 71 — the rung `the_vsix_installs_into_cursor` cannot
/// reach.
///
/// # Why install is not enough
///
/// VSCode does not run `activate` during `--install-extension`, so an
/// extension whose entry module throws on `require` installs cleanly, lists
/// back from `--list-extensions`, and is then inert. Plan 71 §1.2 measured
/// exactly that: the VSIX held seven entries and none of them was a module, so
/// `require("vscode-languageclient/node")` at the top of `extension.js` had
/// nothing to resolve against at any level from `src/` up to `/`.
///
/// The symptom is what made it worth a test rather than a fix alone: with the
/// client gone, the language id, the TextMate grammar and the editor
/// configuration still load, which is precisely `redox.server.enable: false`.
/// A broken install and a deliberately degraded one looked the same.
///
/// # What this checks
///
/// Two rungs, and the names in the failure messages say which is which:
///
/// 1. `vscode-languageclient` is **present in the installed tree** — cheap,
///    and enough to catch a payload regression.
/// 2. `node` **loads the installed entry module**, with a stub `vscode` on
///    `NODE_PATH` standing in for the API the editor injects. This is the one
///    that would have caught the defect as a defect rather than as a missing
///    directory.
#[test]
fn the_installed_extension_can_load_its_entry_module() {
  let dir = scratch("load");
  let vsix = build_vsix(&dir.join("redox.vsix"));

  if !available("cursor") {
    skip(
      "installed-extension load",
      "no `cursor` on PATH — nothing to install into",
    );
    return;
  }

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
  assert!(
    install.status.success(),
    "the VSIX should install before its entry module can be loaded:\n{}{}",
    String::from_utf8_lossy(&install.stdout),
    String::from_utf8_lossy(&install.stderr),
  );

  let installed = ext_dir.join("redox-lang.redox-0.1.0");
  assert!(
    installed.is_dir(),
    "cursor should unpack the extension to {}, and the rest of this test reads \
     that directory",
    installed.display()
  );

  // Rung 1: the closure is there at all.
  let client = installed.join("node_modules/vscode-languageclient");
  assert!(
    client.is_dir(),
    "the installed extension should carry `vscode-languageclient`, because \
     `src/extension.js` requires it at module top level and VSCode does not \
     provide it at runtime. Missing, this extension contributes the language \
     and the TextMate grammar and nothing else — which is indistinguishable \
     from `redox.server.enable: false`. Run `npm ci` in `editors/vscode` so \
     `vsix_payload()` has a closure to walk.\n  looked for {}",
    client.display()
  );

  // Rung 2: node actually loads it.
  if !available("node") {
    skip(
      "entry-module load",
      "no `node` on PATH — the closure was checked, the module was not loaded",
    );
    return;
  }

  // `extension.js` requires `vscode`, which only the editor provides, and
  // `vscode-languageclient` reaches far more of that API than this extension
  // does — it subclasses `CompletionItem` and a dozen others at module load
  // time. A stub enumerating the calls `activate` makes is therefore not
  // enough; the first attempt supplied four and died on
  // `class ProtocolCompletionItem extends code.CompletionItem`.
  //
  // So the stub is **universal**: one self-referential `Proxy` whose every
  // property is itself, and which is callable, constructible and extendable.
  // That is honest about what is being checked — this test asserts that the
  // module graph loads, not that the editor's API behaves.
  let stub = dir.join("stub/node_modules/vscode");
  std::fs::create_dir_all(&stub).expect("should create the vscode stub");
  std::fs::write(
    stub.join("package.json"),
    r#"{"name":"vscode","version":"0.0.0","main":"index.js"}"#,
  )
  .expect("should write the stub manifest");
  std::fs::write(
    stub.join("index.js"),
    "const anything = new Proxy(function () {}, {\n  \
       get: (_t, key) => (key === \"then\" ? undefined : anything),\n  \
       apply: () => anything,\n  \
       construct: () => Object.create(null),\n\
     });\n\
     module.exports = anything;\n",
  )
  .expect("should write the stub module");

  let entry = installed.join("src/extension.js");
  let loaded = Command::new("node")
    .arg("-e")
    .arg(format!(
      "const m = require({:?}); \
       if (typeof m.activate !== 'function') {{ \
         console.error('no activate export'); process.exit(2); \
       }}",
      entry.to_string_lossy()
    ))
    .env("NODE_PATH", dir.join("stub/node_modules"))
    .output()
    .expect("node should run");

  assert!(
    loaded.status.success(),
    "`node` should load the installed `src/extension.js` and find its \
     `activate` export. This is the assertion plan 49's install check could \
     not make: VSCode does not run `activate` during an install, so a module \
     that throws on `require` installs and lists back perfectly.\n{}{}",
    String::from_utf8_lossy(&loaded.stdout),
    String::from_utf8_lossy(&loaded.stderr),
  );
}

/// Write a VSIX from `editors/vscode/`.
///
/// The OPC layout: `[Content_Types].xml`, `extension.vsixmanifest`, and the
/// extension's own files under `extension/`.
pub(super) fn build_vsix(path: &Path) -> PathBuf {
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

  let payload = vsix_payload();

  zip
    .start_file("[Content_Types].xml", options)
    .expect("should start a file");
  zip
    .write_all(content_types(&payload).as_bytes())
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

  for relative in &payload {
    let from = editors_dir().join("vscode").join(relative);
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
/// The extension's own files are **listed**; `node_modules` is **walked**.
///
/// Plan 49 listed all of it, and gave as the reason that a walk "would put
/// `node_modules` in the VSIX". That was true, and it was the defect.
/// `extension.js` requires `vscode-languageclient/node` at module top level,
/// so an extension installed without that closure throws before `activate` is
/// entered, and what survives is the declarative half alone: the language id,
/// the TextMate grammar and the editor configuration. That is byte-identical
/// to `redox.server.enable: false`, which this extension's README calls "the
/// useful degraded mode" — so the failure presented as a supported
/// configuration rather than as an error. Plan 71 §1.2.
///
/// The walk is **scoped to `node_modules`** and so keeps plan 49's intent:
/// nothing else in the directory is walked, and a stray file beside
/// `package.json` still cannot reach the VSIX.
pub(super) fn vsix_payload() -> Vec<String> {
  let mut payload: Vec<String> = [
    "package.json",
    "README.md",
    "language-configuration.json",
    "syntaxes/redox.tmLanguage.json",
    "src/extension.js",
  ]
  .into_iter()
  .map(str::to_string)
  .collect();

  let modules = editors_dir().join("vscode/node_modules");
  let mut found = Vec::new();
  walk(&modules, &modules, &mut found);
  // Sorted, so the VSIX is reproducible: `read_dir` order is the filesystem's.
  found.sort();
  payload.extend(found.into_iter().map(|rel| format!("node_modules/{rel}")));
  payload
}

/// Every file under `at`, as a path relative to `base`.
fn walk(base: &Path, at: &Path, into: &mut Vec<String>) {
  let Ok(entries) = std::fs::read_dir(at) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      walk(base, &path, into);
    } else if let Ok(rel) = path.strip_prefix(base) {
      into.push(rel.to_string_lossy().replace('\\', "/"));
    }
  }
}

/// `[Content_Types].xml`, derived from the payload rather than written down.
///
/// OPC wants a content type for every part. `node_modules` brings `.ts`,
/// `.md`, `.txt`, `.cmd`, `.sh`, `.yml` and `.bnf` alongside the `.js` and
/// `.json` plan 49 knew about, and a dependency bump can bring another — so
/// the four hard-coded `Default` elements became a set computed from the files
/// actually being written. A part with **no extension at all** (every
/// `LICENSE` in the closure) cannot be covered by a `Default`, and gets an
/// `Override`.
fn content_types(payload: &[String]) -> String {
  let mut extensions: Vec<String> = Vec::new();
  let mut overrides: Vec<String> = Vec::new();

  for relative in payload {
    match relative.rsplit_once('.') {
      // A dot in a parent directory's name is not an extension.
      Some((_, extension)) if !extension.contains('/') => {
        let lower = extension.to_ascii_lowercase();
        if !extensions.contains(&lower) {
          extensions.push(lower);
        }
      }
      _ => overrides.push(format!("/extension/{relative}")),
    }
  }
  extensions.sort();
  overrides.sort();

  let mut xml = String::new();
  xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
  xml.push_str("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n");
  // The manifest is not in the payload, and is always present.
  xml.push_str("  <Default Extension=\".vsixmanifest\" ContentType=\"text/xml\" />\n");
  for extension in &extensions {
    if extension == "vsixmanifest" {
      continue;
    }
    xml.push_str(&format!(
      "  <Default Extension=\".{extension}\" ContentType=\"{}\" />\n",
      content_type_of(extension)
    ));
  }
  for part in &overrides {
    xml.push_str(&format!(
      "  <Override PartName=\"{part}\" ContentType=\"text/plain\" />\n"
    ));
  }
  xml.push_str("</Types>\n");
  xml
}

/// A content type per extension.
///
/// An unknown extension is bytes, which is what `application/octet-stream`
/// means and is always safe for a part VSCode does not read.
fn content_type_of(extension: &str) -> &'static str {
  match extension {
    "json" => "application/json",
    "js" | "cjs" | "mjs" => "application/javascript",
    "md" => "text/markdown",
    "xml" | "vsixmanifest" => "text/xml",
    "ts" | "mts" | "cts" | "txt" | "sh" | "cmd" | "yml" | "yaml" | "bnf" | "map" => "text/plain",
    "png" => "image/png",
    "svg" => "image/svg+xml",
    _ => "application/octet-stream",
  }
}

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
