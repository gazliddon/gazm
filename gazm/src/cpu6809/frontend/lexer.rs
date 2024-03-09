use crate::cpu6809::assembler::ISA_DBASE;

use crate::frontend::TokenKind;
use crate::frontend::CommandKind;
use crate::cpukind::CpuKind;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = text.to_lowercase();

    if ISA_DBASE.get_opcode(&text).is_some() {
        TokenKind::OpCode(CpuKind::Cpu6809)
    } else {
        match text.as_str() {
            "setdp" => Command(CommandKind::SetDp),
            _ => Label,
        }
    }
}
