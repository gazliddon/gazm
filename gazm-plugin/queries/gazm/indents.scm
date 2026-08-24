; Struct, macro and repeat bodies indent one level inside their braces.
[
  (struct_def)
  (macro_body)
  (repeat_body)
] @indent.begin

(struct_def "}" @indent.branch)
(macro_body "}" @indent.branch)
(repeat_body "}" @indent.branch)
