;; Highlights!

(string_literal) @string
; (mnemonic (immediate) @tag)
(opcode (immediate) @operator) 

[(dec_num) (hex_num) (bin_num)] @number
(mnemonic) @tag
(comment) @comment @spell
[(label) (local_label)] @type.builtin
[ (incbin) (org) (scope) (fdb) (fcb) (fill) (rmb) (fcc)] @keyword 
(fcb (label) @number)
[(x) (y) (s) (u) (dp) (a) (b) (d) (pc)] @tag
(doc (doc_text) @operator)
(doc) @comment

(ERROR) @error
