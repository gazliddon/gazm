use crate::cpu6809::assembler::ISA_DBASE;

use crate::cpukind::CpuKind;
use crate::frontend::ascii_lowercase;
use crate::frontend::CommandKind;
use crate::frontend::TokenKind;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = ascii_lowercase(text);

    if ISA_DBASE.get_opcode(&text).is_some() {
        TokenKind::CpuOpcode(CpuKind::Cpu6809)
    } else {
        match text.as_ref() {
            "setdp" => Command(CommandKind::SetDp),
            _ => Label,
        }
    }
}
