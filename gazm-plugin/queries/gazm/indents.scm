; Struct and macro bodies indent one level inside their braces.
[
  (struct_def)
  (macro_body)
] @indent.begin

(struct_def "}" @indent.branch)
(macro_body "}" @indent.branch)
