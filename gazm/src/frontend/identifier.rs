use unraveler::tag;

use crate::{
    cpu6800::frontend::lex_identifier as lex6800,
    cpu6809::frontend::lex_identifier as lex6809,
    cpukind::CpuKind,
    frontend::{err_nomatch, get_str, get_text},
};

use super::{PResult, TSpan, TokenKind, COMS};


/// Returns either
/// TokenKind::OpCode
/// TokenKind::Label
/// TokenKind::Command
pub fn lex_identifier(c: CpuKind, text: &str) -> TokenKind {
    use CpuKind::*;
    use TokenKind::{Command, Label, Identifier};
    match c {
        Cpu6809 => lex6809(text),
        Cpu6800 => lex6800(text),
        _ => panic!(),
    }
}

/// Returns either
/// TokenKind::OpCode
/// TokenKind::Label
/// TokenKind::Command
pub fn get_identifier(input: TSpan) -> PResult<TokenKind> {
    // todo needs to handle local labels as well

    use CpuKind::*;

    use TokenKind::{Command, Label, Identifier};

    let c = input.extra().cpu_kind;

    let (rest, matched) = tag(Identifier)(input)?;

    let text = get_str(&matched).to_lowercase();

    if let Some(command) = COMS.get(&text) {
        Ok((rest, Command(*command)))
    } else {
        match c {
            Some(c) =>  {
                let id = lex_identifier(c, &text);
                Ok((rest, id))
            }
            _ => err_nomatch(input)
        }
    }
}

