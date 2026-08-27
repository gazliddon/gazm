use crate::cpukind::CpuKind;
use crate::frontend::ascii_lowercase;
use crate::frontend::TokenKind;

use crate::cpu68000::isa::mnemonic_set;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = ascii_lowercase(text);

    if mnemonic_set().contains(text.as_ref()) {
        TokenKind::CpuOpcode(CpuKind::Cpu68000)
    } else {
        Label
    }
}
