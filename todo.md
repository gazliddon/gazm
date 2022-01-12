# 6809 Assembler
* Change eval so that it will only evaluate InFixEpr, Labels and Numbers
* Split the symbol table out from the evaluators
* Change evaluator to a simplifier
    * Returns an new infix expr or number only

# 6809 Debugger
* Scroll Window States
    * Cursor in bottom scroll zone -> try Scroll Up
    * Cursor in top scrollzone -> try Scroll down
    * Cursor off top of screen by 1-> Cursor to top of screen, try scroll down
    * Cursor off bottom of screen by 1 -> cursor bottom, try scroll 

OffTop(),
OffBottom(is)
InTopScrollZone
InBottomScrollZone


```
struct EdgeDistances {
    to_top: isize,
    to_top_scrollzone: isize,
    to_bottom: isize,
    to_bottom_scrollzone: isize,
    scroll_zone : isize,
    desired_scroll_zone: usize,
    cursor: isize,
}

impl EdgeDistances {
    pub fn new(cursor : isize, lines: usize, scroll_zone : usize) -> Self {
        let lines = lines as isize;
        let mut scroll_zone = desired_scroll_zone as isize;

        // do we have space for a scroll zone?

        if lines - (scroll_zone * 2) < 1 {
            scroll_zone = 0;
        }

        let to_bottom = (lines -1) - cursor;

        EdgeDistances {
            to_top: cursor,
            to_top_scrollzone: cursor - scroll_zone,
            to_bottom: to_bottom,
            to_bottom_scrollzone: to_bottom - 3,
            scroll_zone: scroll_zone as usize,
            desired_scroll_zone,
            cursor,
        }

    }
} 

```

Wordle 205 5/6

⬛🟨🟩⬛⬛
⬛⬛🟩🟩⬛
⬛⬛⬛⬛⬛
⬛🟩⬛⬛🟩
🟩🟩🟩🟩🟩



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
