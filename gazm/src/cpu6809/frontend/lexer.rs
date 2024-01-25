use crate::cpu6809::assembler::ISA_DBASE;
use crate::frontend::TokenKind;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = text.to_lowercase();

    if ISA_DBASE.get_opcode(&text).is_some() {
        TokenKind::OpCode
    } else {
        match text.as_str() {
            "setdp" => Command,
            _ => Label,
        }
    }
}
