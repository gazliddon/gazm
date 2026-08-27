;; Highlights!

;; Strings
(string_literal) @string

;; Directives & Commands
(importer) @keyword.import

[(incbin) (org) (section) (scope) (fdb) (fcb) (fill) (rmb) (fcc) (include)
 (writebin) (setdp) (bsz) (zmb) (zmd) (exec_addr)] @keyword.directive

; MadMac / JRISC directives (dot spellings, REGEQU, MACRO/ENDM)
[(madmac_simple) (madmac_decl) (madmac_dc) (madmac_macro) (endm) (regequ)] @keyword.directive

; MadMac register equates and macro names read as labels; the directive word
; itself is an anonymous token inside the parent rule, so these come AFTER
; the directive captures above to let the name win over the keyword colour.
(regequ (label) @label)
(regequ (scoped_label) @label)
(regequ (local_label) @label)
(madmac_macro (label) @label)
(madmac_macro (scoped_label) @label)
(madmac_macro (local_label) @label)

; `repeat` is not a reserved token (REPEAT stays a usable symbol), so it is
; matched as a plain identifier in statement position and highlighted via
; the keyword field instead.
(repeat keyword: (label) @keyword.directive)

; Control-flow keywords, same treatment as `repeat`.
(if_statement keyword: (if_keyword) @keyword.directive)
(else_clause keyword: (else_keyword) @keyword.directive)
(while_statement keyword: (while_keyword) @keyword.directive)
(for_statement keyword: (for_keyword) @keyword.directive)
(break_statement keyword: (break_keyword) @keyword.directive)
(continue_statement keyword: (continue_keyword) @keyword.directive)

;; Structs & Macros
(elem_type) @type.builtin

;; Compile-time function calls
(call_expression function: (label) @function.call)
(call_expression function: (scoped_label) @function.call)

;; Mnemonics & Opcodes
(mnemonic) @function.builtin

;; Labels & Identifiers
(local_label) @label
(label) @label
(scoped_label) @label

;; Scope & Namespaces
(scope (label) @module)
(scope (local_label) @module)
(import_group scope: (label) @module)
(import_group name: (label) @label)

;; Punctuation
["{" "}" "[" "]"] @punctuation.bracket
["," "::" ":"] @punctuation.delimiter

;; Numbers
[(dec_num) (hex_num) (bin_num) (float_num)] @number

;; Operators & Operands
(operand (immediate) @operator)

;; Registers
(fcb (label) @number)
[(x) (y) (s) (u) (dp) (a) (b) (d) (pc)] @variable.builtin

; Registers that lex as plain identifiers on their CPU: 68000 dN/aN/SP/SR/
; CCR/USP, JRISC r0-r31, z80 HL/BC/DE/AF/IX/IY, 6800/6502 SP.  Must come
; after `(label) @label` so the register colour wins.  The outer parens are
; required: a predicate only attaches to the node pattern when both are
; wrapped in one parenthesised pattern.
((label) @variable.builtin
  (#match? @variable.builtin
    "^(a|b|x|y|s|u|d|sp|pc|sr|ccr|usp|d[0-7]|a[0-7]|r([0-9]|[12][0-9]|3[01])|hl|bc|de|af|ix|iy)$"))

;; Comments & Docs
(comment) @comment @spell
(doc (doc_text) @comment.documentation)
(doc) @comment.documentation
(long_doc) @comment.documentation

;; Errors
(ERROR) @error


; Compile-time checks and logging
(assert_statement keyword: (assert_keyword) @keyword.directive)
(log_statement keyword: (log_keyword) @keyword.directive)
