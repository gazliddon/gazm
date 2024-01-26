use crate::cpu6800::DBASE;

use crate::frontend::CommandKind;
use crate::frontend::TokenKind;

pub fn lex_identifier(text: &str) -> TokenKind {
    use TokenKind::*;

    let text = text.to_lowercase();

    if DBASE.get_opcode(&text).is_some() {
        TokenKind::OpCode
    } else {
        Label
    }
}
