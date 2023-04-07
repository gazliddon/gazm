# AST Navigation


```rust
    let x = 10;
```
# Todo

# Todo
* Don't remove includes from AST

## Misc
* Add variable scoping into tokenizer

## Require
- [ ] Change scope syntax to be in a block?

## Misc
- [x] Make all line counts start from line 0, only > 1 for displaying etc
- [ ] Text position -> Ast Item
- [x] Move FMT in gazm

## LSP
* Get document changes
    - [*] Initial implementation register only for complete doc changes
    - [*] Add interface to incorporate change of entire doc
    - [ ] Longer term add in local changes for doc
    - [ ] Send back errors

* Inline errors
    - [*] Recompile on doc open
    - [*] Recompile on doc change
    - [ ] Recompile work space changes

* [*] Show references
* Show symbol value
    * On Shift-K on references
    * Next to definitions

# Fix
* Return a proper range for goto defintion
