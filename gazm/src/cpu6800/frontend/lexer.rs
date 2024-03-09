
use emu6800::cpu_core::DBASE;

use crate::frontend::CommandKind;
use crate::frontend::TokenKind;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = text.to_lowercase();

    if DBASE.get_opcode(&text).is_some() {
        TokenKind::OpCode(crate::cpukind::CpuKind::Cpu6800)
    } else {
        Label
    }
}
