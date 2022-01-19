# 6809 Assembler
* Try out in emu
    * Write a map file with SLB data
    * Write binary
    * Load and run

* macros
* structs
* Add in forced 5 bit offset as per as6809
* Other unary operators
* Allow for multiple errors (maxerr amount)
* change equ * to assign PC
* Add in PCR to flag PC relative
* Back bin writer with a memory block

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

(fn stilob[a b c]
    (asm #[
            JSR    GETOB            ;GET AN OBJECT
            LDD    #0               ;VELOCITY
            STD    OXV,X
            STD    OYV,X
            LDD    #b              ;X
            STD    OX16,X
            LDD    #c
            STD    OY16,X
            LDD    #a              ;PICTURE
            STD    OPICT,X
            STX    OPTR             ;AND APPEND TO LIST
    ]))

(def task (struct {a : byte, b: word, c : dword}))

struct Task {
    a : byte,
    b : word,
    c : dword,
}

local task_table = Task[100]

fn stilob(a,b,c) {
    asm {
            JSR    GETOB            ;GET AN OBJECT
            LDD    #0               ;VELOCITY
            STD    OXV,X
            STD    OYV,X
            LDD    #b              ;X
            STD    OX16,X
            LDD    #c
            STD    OY16,X
            LDD    #a              ;PICTURE
            STD    OPICT,X
            STX    OPTR             ;AND APPEND TO LIST
    }
}


asm {
STILOB  MACRO  \1,\2,\3
        JSR    GETOB            ;GET AN OBJECT
        LDD    #0               ;VELOCITY
        STD    OXV,X
        STD    OYV,X
        LDD    #\2              ;X
        STD    OX16,X
        LDD    #\3
        STD    OY16,X
        LDD    #\1              ;PICTURE
        STD    OPICT,X
        STX    OPTR             ;AND APPEND TO LIST
        ENDM 
}





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
