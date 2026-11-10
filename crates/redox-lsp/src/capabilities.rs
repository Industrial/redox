/// The legend the server's token indices refer into.
pub(crate) mod token_types;

use crate::capabilities::token_types::token_types;
use lsp_types::{
  OneOf, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
  SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
  TextDocumentSyncKind, WorkDoneProgressOptions,
};

/// What this server tells the editor it can do.
///
/// # Only what is implemented
///
/// Plan 40's criterion 1, and it is a real rule rather than tidiness: an editor
/// told a capability exists **stops looking for another source for it**. A
/// server that advertises completion and returns nothing is worse than one
/// that advertises none, because the editor's own word-based fallback is
/// switched off by the advertisement.
///
/// So the list is exactly four, and `capabilities_match_the_handlers` in the
/// test suite is the assertion that it stays that way.
///
/// # Full-document sync
///
/// Plan 40 decision 5. The parser is fast enough that incremental sync is a
/// complication without a measurement — and row 44's benchmarks are where that
/// stops being an assertion. Full sync also means the document store holds
/// exactly what the editor holds, so a diagnostic's span is an offset into the
/// text the user can see.
pub fn capabilities() -> ServerCapabilities {
  ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
    document_symbol_provider: Some(OneOf::Left(true)),
    document_formatting_provider: Some(OneOf::Left(true)),
    semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
      SemanticTokensOptions {
        work_done_progress_options: WorkDoneProgressOptions::default(),
        legend: SemanticTokensLegend {
          token_types: token_types(),
          token_modifiers: Vec::new(),
        },
        range: None,
        full: Some(SemanticTokensFullOptions::Bool(true)),
      },
    )),
    ..ServerCapabilities::default()
  }
}
