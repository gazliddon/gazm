# Todo

* How do I make a vim 6809 assembly theme?
* Make instruction dbase generate at compile time
* Make list of errors to correct in disassembly

# 6809 Assembler
https://github.com/Geal/nom
https://github.com/Geal/nom/blob/master/examples/s_expression.rs

## Very Rough Grammar
```
hexdigit [0-9a-fA-F]
number [0-9]+, $[hexdigit]+, 0x[hexdigit]+
opcode = [...]

label_start = [_a-zA-Z]
label = label_start(labelstart|[0-8])*
loc_lobel = @label

immediate = #value
for_direct = <value
reg = [abxydus]
reg_list = reg (,reg)*

```

# Vim Plugin
* https://github.com/tpope/vim-scriptease
* https://learnvimscriptthehardway.stevelosh.com/chapters/42.html


## Directory Layout
    ~/.vim/colors/ colourschemes
    ~/.vim/plugin/ files in here will be run once when vim starts
    ~/.vim/ftdetect/ as above - should have commands to detect ft and run autocmds
    ~/.vim/ftplugin/ will load according to buffer ftype. set ftype=derp will load derp.vim
    ~/.vim/indent/ loads as per above, contain indenting info
    ~/.vim/compiler/
    ~/.vim/after/
    ~/.vim/autoload/
    ~/.vim/doc/

I need to do:
* plugin
* ftdetect
* ftplugin
* indent
* syntax

**What about ~/.vim/syntax directory?**
