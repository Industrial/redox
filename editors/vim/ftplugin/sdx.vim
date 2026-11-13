" Voltaic buffer settings for Vim. Plan 49 leaf 2.

if exists('b:did_ftplugin')
  finish
endif
let b:did_ftplugin = 1

setlocal expandtab
setlocal shiftwidth=2
setlocal softtabstop=2
setlocal commentstring=--\ %s
setlocal comments=:--\ \|,:--\ !,:--

" Voltaic is off-side-rule, so `autoindent` keeping the previous line's
" indentation is the right behaviour and `smartindent` — which reindents on
" seeing a brace — is actively wrong here.
setlocal autoindent
setlocal nosmartindent
setlocal nocindent

" Formatting is a command rather than `formatprg`, and that is forced rather
" than chosen. `formatprg` pipes the buffer through a filter on stdin; `voltaic
" fmt` has no `--stdin` — it formats files in place and refuses to write a file
" it could not parse (plan 34 criterion 8). So the command writes the buffer,
" formats the file, and reloads it, which is the same safety property.
command! -buffer VoltaicFmt call s:VoltaicFmt()

function! s:VoltaicFmt() abort
  if &modified
    write
  endif
  let l:view = winsaveview()
  let l:out = system('voltaic fmt ' . shellescape(expand('%:p')))
  if v:shell_error != 0
    echohl ErrorMsg | echomsg substitute(l:out, '\n\+$', '', '') | echohl None
    return
  endif
  silent edit!
  call winrestview(l:view)
endfunction

let b:undo_ftplugin = 'setlocal expandtab< shiftwidth< softtabstop< '
      \ . 'commentstring< comments< autoindent< smartindent< cindent< '
      \ . '| delcommand VoltaicFmt'
