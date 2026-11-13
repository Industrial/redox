# -*- coding: utf-8 -*-
"""Plan 78 batch 4: `capabilities` and `answer`.

**`capabilities` is the first ported use of the record-update form.** The Rust
ends `..ServerCapabilities::default()`, and doc 18 §6 lowers a **lowercase**
head to a mut-bind plus assignments — because Rust's `X { f: v, ..base }` needs
the type's *path* and the source gives only the value's name. So the default is
bound and the fields are set on it, and the emitted Rust contains no `..` at
all.

**`answer` is where batch 1's `cast` change lands.** Each arm ascribes the
params type it wants instead of writing a turbofish, which is the replacement
doc 03 §18 names.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('capabilities', '''
export capabilities : ServerCapabilities
capabilities =
  -- **The record-update form**, where the Rust wrote
  -- `..ServerCapabilities::default()`. Doc 18 §6 lowers a lowercase head to a
  -- mut-bind plus assignments, because Rust's own `X { f: v, ..base }` needs
  -- the type's path and the source gives only the value's name. The emitted
  -- Rust therefore contains no `..`.
  base = ServerCapabilities::default ()
  base {
    text_document_sync = Some (TextDocumentSyncCapability::Kind TextDocumentSyncKind::FULL),
    document_symbol_provider = Some (OneOf::Left true),
    document_formatting_provider = Some (OneOf::Left true),
    semantic_tokens_provider = Some (SemanticTokensServerCapabilities::SemanticTokensOptions (SemanticTokensOptions {
      work_done_progress_options = WorkDoneProgressOptions::default (),
      legend = SemanticTokensLegend {
        token_types = token_types (),
        token_modifiers = Vec::new (),
      },
      range = None,
      full = Some (SemanticTokensFullOptions::Bool true),
    })),
  }
''')

write('serve/answer', '''
import lsp_server::ExtractError
import lsp_types::{DocumentFormattingParams, DocumentSymbolParams, SemanticTokensParams}

export(crate) answer : &Request -> &Documents -> Response
answer request documents =
  id = .clone request.id
  method = .as_str request.method

  -- An ascription per arm, where the Rust wrote `cast::<DocumentSymbolRequest>`.
  -- Doc 03 §18 has no turbofish and names ascription as the replacement; `cast`
  -- takes the method as a value for the reason its own doc comment records.
  if method == DocumentSymbolRequest::METHOD
    got : Result<DocumentSymbolParams, ExtractError<Request>> = cast request DocumentSymbolRequest::METHOD
    return reply id got (\\params -> symbols (.unwrap_or_default (.text (&params.text_document.uri) documents)))

  if method == Formatting::METHOD
    got : Result<DocumentFormattingParams, ExtractError<Request>> = cast request Formatting::METHOD
    return reply id got (\\params -> formatting (.unwrap_or_default (.text (&params.text_document.uri) documents)))

  if method == SemanticTokensFullRequest::METHOD
    got : Result<SemanticTokensParams, ExtractError<Request>> = cast request SemanticTokensFullRequest::METHOD
    return reply id got (\\params -> semantic_tokens (.unwrap_or_default (.text (&params.text_document.uri) documents)))

  -- A method this server did not advertise gets a `null` **result** rather
  -- than an error. The protocol allows either, and `null` is the kinder
  -- one: an error reply shows up in the user's log as though they had done
  -- something wrong, when it was the editor that asked.
  Response {
    id,
    response_result = Ok serde_json::Value::Null,
  }
''')

print('plan 78 batch 4: two files')
