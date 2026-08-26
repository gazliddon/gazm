use crate::cpu_z80::assembler::ISA_DBASE;
use crate::cpukind::CpuKind;
use crate::frontend::ascii_lowercase;
use crate::frontend::TokenKind;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = ascii_lowercase(text);

    if ISA_DBASE.get_info(&text.to_ascii_uppercase()).is_some() {
        TokenKind::CpuOpcode(CpuKind::CpuZ80)
    } else {
        Label
    }
}
