" Vim syntax file for Shelly `.ly` scripts
if exists("b:current_syntax")
  finish
endif

syntax case match

syntax keyword shellyKeyword if elif else end repeat while fn global export import include exit
syntax keyword shellyCommand shellout shellin ask command append prepend remove
syntax keyword shellyBoolean true false
syntax keyword shellySpecial first last contained

syntax match shellyVariable /\$[A-Za-z][A-Za-z0-9_]*/
syntax match shellyConstant /\$\$[A-Z][A-Z0-9_]*/
syntax match shellyNumber /\v<[-+]?[0-9]+(\.[0-9]+)?>/
syntax match shellyOperator /\v(->|==|!=|>=|<=|>|<|\+|-|\*|\/|=|\.\.)/
syntax match shellyListSelector /\[[^\]]*\]/ contains=shellyVariable,shellyConstant,shellyNumber,shellyString,shellyBoolean,shellyOperator,shellySpecial

syntax region shellyString start=/"/ skip=/\\"/ end=/"/ contains=shellyInterpolation
syntax match shellyInterpolation /\$\$[A-Z][A-Z0-9_]*/ contained
syntax match shellyInterpolation /\$[A-Za-z][A-Za-z0-9_]*\(\[[^\]]*\]\)\?/ contained

syntax region shellyComment start=/#/ end=/#/ contains=NONE

highlight default link shellyKeyword Keyword
highlight default link shellyCommand Function
highlight default link shellyBoolean Boolean
highlight default link shellyVariable Identifier
highlight default link shellyConstant Constant
highlight default link shellyNumber Number
highlight default link shellyOperator Operator
highlight default link shellyString String
highlight default link shellyInterpolation Special
highlight default link shellyListSelector Special
highlight default link shellySpecial Special
highlight default link shellyComment Comment

let b:current_syntax = "shelly"
