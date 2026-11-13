# -*- coding: utf-8 -*-
"""Plan 78 batch 3: `serve`, `Documents` and `absorb`.

`Documents` is the first ported type whose `new` goes through `Default`, and
`absorb` the first to deserialise into an ascribed binding three times — which
is the shape a turbofish would otherwise take, so it reads as the replacement
rather than as a workaround.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('serve/documents', '''
-- The `data` declaration is **not** repeated here: the translator left it in
-- the head, above the `-- TODO` marker, because the marker sits at the `impl`.
-- A body that restates it is `E0428: the name Documents is defined multiple
-- times` — the mistake plan 73 recorded for `reportable.sdx`'s two
-- `data Reportable`.
instance Documents
  -- | An empty store, holding no open document.
  export(crate) new : Self
  new = Documents::default ()

  -- | Record a document's current text, replacing any previous version.
  export(crate) set : Uri -> String -> &mut self -> ()
  set uri text self =
    .insert uri text self.open
    ()

  -- | Forget a closed document.
  export(crate) close : &Uri -> &mut self -> ()
  close uri self =
    .remove uri self.open
    ()

  -- | A document's text, or `None` if the editor never opened it.
  -- |
  -- | `None` is not an error to report. An editor may send a request for a
  -- | document it has already closed — the two cross on the wire — so a server
  -- | that treated it as a protocol violation would log noise at exactly the
  -- | moment the user did nothing wrong.
  export(crate) text : &Uri -> &self -> Option<&str>
  text uri self = .map String::as_str (.get uri self.open)

  -- No `len` or `is_empty`. Both were written for "the tests that assert the
  -- store is maintained", and no test wants them: `didOpen` followed by a
  -- request that answers from the document is already that assertion, and it
  -- asserts the behaviour rather than the bookkeeping. An accessor justified
  -- by a test that does not exist is dead code with an alibi.
''')

write('serve/absorb', '''
export(crate) absorb : &Notification -> &mut Documents -> Option<Uri>
absorb notification documents =
  method = .as_str notification.method
  -- An ascribed binding per arm, where the Rust annotated the `let`. That is
  -- the same spelling in both languages; `serde_json::from_value` is generic
  -- over its output and the type has to be said somewhere.
  if method == DidOpenTextDocument::METHOD
    params : lsp_types::DidOpenTextDocumentParams = (.ok (serde_json::from_value (.clone notification.params)))?
    uri = .clone params.text_document.uri
    .set (.clone uri) params.text_document.text documents
    return Some uri

  if method == DidChangeTextDocument::METHOD
    params : lsp_types::DidChangeTextDocumentParams = (.ok (serde_json::from_value (.clone notification.params)))?
    -- Full sync, so the last change carries the whole document. Taking the
    -- last rather than the first matters: the protocol allows a batch, and
    -- with full sync every entry is a complete replacement, so the earlier
    -- ones are superseded.
    -- `last_change`, not `final`: **`final` is reserved in *Rust*** and not in
    -- Voltaic, so this is the keyword collision running the other way — a name
    -- Voltaic accepts and rustc rejects, reported against the generated file.
    -- What a binding has to avoid is the **union** of both keyword sets, and
    -- Rust reserves several words it does not use: `final`, `become`, `priv`,
    -- `typeof`, `unsized`, `virtual`, `abstract`, `override`, `box`.
    last_change = (.last (.into_iter params.content_changes))?
    uri = .clone params.text_document.uri
    .set (.clone uri) last_change.text documents
    return Some uri

  if method == DidCloseTextDocument::METHOD
    params : lsp_types::DidCloseTextDocumentParams = (.ok (serde_json::from_value (.clone notification.params)))?
    .close (&params.text_document.uri) documents
    return None

  None
''')

write('serve', '''
export serve : &Connection -> Result<(), String>
serve connection =
  mut documents = Documents::new ()
  for message in &connection.receiver
    case message of
      Message::Request request ->
        stopping = (.map_err (\\e -> .to_string e) (.handle_shutdown (&request) connection))?
        if stopping
          return Ok ()
        response = answer (&request) (&documents)
        (.map_err (\\e -> .to_string e) (.send (Message::Response response) connection.sender))?
      Message::Notification notification ->
        case absorb (&notification) (&mut documents) of
          Some uri ->
            (publish connection (&uri) (&documents))?
          None -> ()
      -- A response is the editor answering *our* request, and this server
      -- makes none. Ignored rather than logged: an editor is free to send
      -- one, and a server that complained would be complaining about
      -- conformant behaviour.
      Message::Response _ -> ()
  Ok ()
''')

print('plan 78 batch 3: three files')
