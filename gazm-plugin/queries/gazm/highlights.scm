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
(import_group scope: (scoped_label) @module)

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
