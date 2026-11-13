-- Voltaic support for Neovim. Plan 49 leaf 2.
--
-- Three things, none of which this file implements: plan 41's tree-sitter
-- grammar for highlighting, plan 40's `voltaic-lsp` for diagnostics and symbols,
-- and `voltaic fmt` for formatting. The file is wiring.
--
-- It needs no plugin manager and no `nvim-treesitter`. Registering a parser is
-- `vim.treesitter.language.add`, which is core since nvim 0.9, and depending on
-- a plugin would make this repository's gate fetch from the network — which
-- plan 49 §1.3 already declined once over `vsce`.
--
-- The queries are **read from the grammar directory at setup time** rather than
-- copied into `queries/voltaic/`. nvim's runtimepath wants `queries/<lang>/<name>.scm`
-- and the grammar ships `queries/<name>.scm`, so the layouts do not line up —
-- and a copy would be a second set of queries to keep in step with plan 41's.
-- `vim.treesitter.query.set` takes the text, so there is no need for either.

local M = {}

--- Read a file, or nil if it is not there.
---
--- Not an error: a checkout with no parser built yet should still get filetype
--- detection and the language server.
local function read(path)
  local handle = io.open(path, 'r')
  if not handle then
    return nil
  end
  local text = handle:read('*a')
  handle:close()
  return text
end

--- The three queries plan 41 ships, in the order nvim asks for them.
M.queries = { 'highlights', 'injections', 'locals' }

--- Wire up Voltaic.
---
--- @param opts table|nil
---   grammar_dir  path to `tree-sitter-voltaic/` — required for highlighting
---   parser       path to the compiled parser (`voltaic.so`); defaults to
---                `<grammar_dir>/voltaic.so`
---   server       the `voltaic-lsp` command; defaults to `{ 'voltaic-lsp' }`
---   lsp          set false to skip the language server
---   format       set false to skip the formatter wiring
function M.setup(opts)
  opts = opts or {}

  -- `.sdx` is Voltaic. `.xrs` is **ordinary Rust** — that is the whole premise of
  -- the compiler, so the generated file gets Rust's own support rather than a
  -- second Voltaic mode that would highlight it wrongly.
  vim.filetype.add({
    extension = {
      sdx = 'voltaic',
      xrs = 'rust',
    },
  })

  if opts.grammar_dir then
    local parser = opts.parser or (opts.grammar_dir .. '/voltaic.so')
    local ok = pcall(vim.treesitter.language.add, 'voltaic', { path = parser })
    if ok then
      for _, name in ipairs(M.queries) do
        local text = read(opts.grammar_dir .. '/queries/' .. name .. '.scm')
        if text then
          vim.treesitter.query.set('voltaic', name, text)
        end
      end
      vim.api.nvim_create_autocmd('FileType', {
        pattern = 'voltaic',
        callback = function()
          pcall(vim.treesitter.start)
        end,
      })
    end
  end

  if opts.lsp ~= false then
    local cmd = opts.server or { 'voltaic-lsp' }
    vim.api.nvim_create_autocmd('FileType', {
      pattern = 'voltaic',
      callback = function(args)
        -- `vim.lsp.start` reuses a client whose name and root match, so one
        -- server serves every buffer in a project.
        vim.lsp.start({
          name = 'voltaic-lsp',
          cmd = cmd,
          root_dir = vim.fs.root(args.buf, { 'Cargo.toml', '.git' }) or vim.fn.getcwd(),
        }, { bufnr = args.buf })
      end,
    })
  end

  if opts.format ~= false then
    vim.api.nvim_create_autocmd('FileType', {
      pattern = 'voltaic',
      callback = function()
        -- `gq` goes through the server, which is plan 40's formatter hook onto
        -- plan 34.
        --
        -- There is deliberately no `formatprg` fallback. `formatprg` pipes the
        -- buffer through a filter on stdin, and `voltaic fmt` has no `--stdin`:
        -- it formats files in place, and refuses to write a file it could not
        -- parse (plan 34 criterion 8), which is a safety property a stdin
        -- filter could not offer. Setting `formatprg = 'voltaic fmt'` would send
        -- the buffer to a command that reads none of it. `editors/vim/` has the
        -- same constraint and solves it with `:VoltaicFmt` on the file.
        vim.bo.formatexpr = 'v:lua.vim.lsp.formatexpr()'
        vim.bo.commentstring = '-- %s'
        vim.bo.expandtab = true
        vim.bo.shiftwidth = 2
        vim.bo.softtabstop = 2
      end,
    })
  end
end

return M
