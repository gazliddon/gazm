if exists("b:current_syntax")
  finish
endif

syn clear

" Case is not important
syn case ignore

" Keywords have to start with a character
" set isk=a-z,A-Z

set isk=a-z,A-Z,48-57,',.,_

" Highlight from the start of the file
syn sync fromstart

syn keyword mc6809Inst abx adca adcb adda addb addd
syn keyword mc6809Inst anda andb andcc asr asra asrb
syn keyword mc6809Inst beq bge bgt bhi bhs bcc
syn keyword mc6809Inst bita bitb ble blo bcs bls
syn keyword mc6809Inst blt bmi bne bpl bra brn
syn keyword mc6809Inst bsr bvc bvs clr clra clrb
syn keyword mc6809Inst cmpa cmpb cmpd cmps cmpu cmpx
syn keyword mc6809Inst cmpy com coma comb cwai daa
syn keyword mc6809Inst dec deca decb eora eorb exg
syn keyword mc6809Inst inc inca incb jmp jsr lbeq
syn keyword mc6809Inst lbge lbgt lbhi lbhs lbcc lble
syn keyword mc6809Inst lblo lbcs lbls lblt lbmi lbne
syn keyword mc6809Inst lbpl lbra lbrn lbsr lbvc lbvs
syn keyword mc6809Inst lda ldb ldd lds ldu ldx
syn keyword mc6809Inst ldy leas leau leax leay lsl
syn keyword mc6809Inst asl lsla asla lslb aslb lsr
syn keyword mc6809Inst lsra lsrb mul neg nega negb
syn keyword mc6809Inst nop ora orb orcc pshs pshu
syn keyword mc6809Inst puls pulu rol rola rolb ror
syn keyword mc6809Inst rora rorb rti rts sbca sbcb
syn keyword mc6809Inst sex sta stb std sts stu
syn keyword mc6809Inst stx sty suba subb subd swi
syn keyword mc6809Inst swi2 swi3 sync tfr tst tsta
syn keyword mc6809Inst tstb

"label, start of line
syn match mc6809Label "\v^[@a-zA-Z_0-9_]+"

"
syn match mc6809comment ";.*$"
syn match mc6809number "\$[0-9a-fA-F]\+"
syn match mc6809number "\v0x[0-9a-fA-F]+"
syn match mc6809number "\<\d\+\>"

syn keyword mc6809Directive fdb equ include fill org


" Strings
syn region mc6809String start=/"/ skip=/\\"/ end=/"/ oneline
syn region mc6809String start=/'/ end=/'/ oneline


" Define the default highlighting.
" For version 5.7 and earlier: only when not done already
" For version 5.8 and later: only when an item doesn't have highlighting yet
if version >= 508 || !exists("did_z80_syntax_inits")
if version < 508
let did_z80_syntax_inits = 1
command -nargs=+ HiLink hi link <args>
else
command -nargs=+ HiLink hi def link <args>
endif

HiLink mc6809Inst Identifier
HiLink mc6809comment Comment
HiLink mc6809Label Type
HiLink mc6809Directive PreProc
HiLink mc6809String String
HiLink mc6809number Number

delcommand HiLink
endif

let b:current_syntax = "mc6809"

