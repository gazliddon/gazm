use unraveler::tag;

use crate::{
    cpu6800::frontend::lex_identifier as lex6800,
    cpu6809::frontend::lex_identifier as lex6809,
    cpukind::CpuKind,
    frontend::{get_str, get_text},
};

use super::{PResult, TSpan, TokenKind, COMS};

pub fn lex_identifier(c: CpuKind, text: &str) -> TokenKind {
    use CpuKind::*;
    use TokenKind::{Command, Label, TempIdentifier};
    match c {
        Cpu6809 => lex6809(text),
        Cpu6800 => lex6800(text),
        _ => panic!(),
    }
}

pub fn parse_identifier(c: CpuKind, input: TSpan) -> PResult<TokenKind> {
    use CpuKind::*;
    use TokenKind::{Command, Label, TempIdentifier};

    let (rest, matched) = tag(TempIdentifier)(input)?;

    let text = get_str(&matched).to_lowercase();

    if let Some(command) = COMS.get(&text) {
        Ok((rest, Command(*command)))
    } else {
        let id = lex_identifier(c, &text);
        Ok((rest, id))
    }
}
