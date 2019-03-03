" Vim syntax file
" Language: atto

if version < 600
  syntax clear
elseif exists("b:current_syntax")
  finish
endif

" keywords
syn keyword attoKeyword     fn nextgroup=attoFnName skipwhite
syn keyword attoKeyword     is
syn keyword attoCond        if
syn keyword attoBoolean		true false
syn keyword attoBuiltIn     __head __tail __fuse __pair
syn keyword attoBuiltIn     __litr __str __words __input __print
syn keyword attoBuiltIn     __eq __add __neg __mul __div __rem
syn keyword attoBuiltIn     __less __lesseq

" matches
syn match attoNumber        "\<\d\>" display
syn match attoNumber        "\<[1-9]\d\+\>" display
syn match attoFnName        "\%([^[:cntrl:][:space:][:digit:]]\|_\)\%([^[:cntrl:][:punct:][:space:]]\|_\)*" display contained
syn match attoComment       "#\s\".*\""
syn match attoStringCont    "/\\\n\s*/" display contained 

" regions
syn region  attoString start=+"+ end=+"+ contains=attoStringCont

" links
hi def link attoKeyword     Keyword
hi def link attoComment     Comment
hi def link attoFnName      Function
hi def link attoCond        Conditional
hi def link attoBoolean     Boolean
hi def link attoBuiltIn     Special
hi def link attoString      String
hi def link attoNumber      Number

let b:current_syntax = "atto"

