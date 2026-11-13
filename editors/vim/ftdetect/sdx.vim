" Plan 49 leaf 2.
"
" `.sdx` is Voltaic. `.xrs` is **ordinary Rust** — that is the compiler's whole
" premise, so the generated file gets Vim's own Rust support rather than a
" second Voltaic mode that would highlight it wrongly.
autocmd BufRead,BufNewFile *.sdx setfiletype sdx
autocmd BufRead,BufNewFile *.xrs setfiletype rust
