;; Highlights!

;; Strings
(string_literal) @string

;; Directives & Commands
(importer) @keyword.import

[(incbin) (org) (section) (scope) (fdb) (fcb) (fill) (rmb) (fcc) (include)
 (writebin) (setdp) (bsz) (zmb) (zmd) (exec_addr)] @keyword.directive

; `repeat` is not a reserved token (REPEAT stays a usable symbol), so it is
; matched as a plain identifier in statement position and highlighted via
; the keyword field instead.
(repeat keyword: (label) @keyword.directive)

;; Structs & Macros
(elem_type) @type.builtin

;; Mnemonics & Opcodes
(mnemonic) @function.builtin

;; Labels & Identifiers
(local_label) @label
(label) @label

;; Scope & Namespaces
(scope (label) @module)
(scope (local_label) @module)
(import_group scope: (label) @module)
(import_group name: (label) @label)

;; Punctuation
["{" "}" "[" "]"] @punctuation.bracket
["," "::" ":"] @punctuation.delimiter

;; Numbers
[(dec_num) (hex_num) (bin_num)] @number

;; Operators & Operands
(operand (immediate) @operator)

;; Registers
(fcb (label) @number)
[(x) (y) (s) (u) (dp) (a) (b) (d) (pc)] @variable.builtin

;; Comments & Docs
(comment) @comment @spell
(doc (doc_text) @comment.documentation)
(doc) @comment.documentation
(long_doc) @comment.documentation

;; Errors
(ERROR) @error

