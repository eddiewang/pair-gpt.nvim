if exists('g:loaded_pair_gpt') | finish | endif

let s:save_cpo = &cpo
set cpo&vim

" command! -range ChatGPTWrite lua require'pair-gpt'.write()
" command! -range ChatGPTRefactor lua require'pair-gpt'.refactor()
command! -range ChatGPTExplain lua require'pair-gpt'.explain()
command! -range ChatGPTWalkthrough lua require'pair-gpt'.walkthrough()

let &cpo = s:save_cpo
unlet s:save_cpo

let g:loaded_pair_gpt = 1
