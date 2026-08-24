use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use strum::IntoEnumIterator;
use unraveler::{kind, Collection};

use crate::{
    cpu6800::frontend::lex_identifier as lex6800,
    cpu6809::frontend::lex_identifier as lex6809,
    cpukind::CpuKind,
    frontend::{err_nomatch, get_str, get_text},
};

use super::{CommandKind, PResult, TSpan, TokenKind};

/// Target-independent assembler commands, indexed by their lowercase name.
pub static COMS: LazyLock<HashMap<String, CommandKind>> = LazyLock::new(|| {
    CommandKind::iter()
        .map(|command| (command.keyword_name().to_string(), command))
        .collect()
});

/// Lowercase ASCII text without allocating when it is already lowercase.
pub fn ascii_lowercase(text: &str) -> Cow<'_, str> {
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(text.to_ascii_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

pub fn command_kind(text: &str) -> Option<&'static CommandKind> {
    let lowered = ascii_lowercase(text);
    COMS.get(lowered.as_ref())
}

/// Match a single identifier token whose text equals `kw` (case-insensitive)
/// and return the consumed span.
///
/// Keywords are only special where a parser chooses to match them — at
/// statement level, or after a label colon for `equ` — never in the token
/// stream itself. So user symbols with the same spelling (`REPEAT`,
/// `LOOP`, ...) keep working everywhere else, and new keywords can never
/// collide with existing code.
pub fn keyword(kw: &str) -> impl FnMut(TSpan) -> PResult<TSpan> + '_ {
    move |input| {
        let (rest, matched) = kind(TokenKind::Identifier)(input)?;
        if ascii_lowercase(get_str(&matched)) == kw {
            Ok((rest, matched))
        } else {
            err_nomatch(input)
        }
    }
}

/// Returns either
/// TokenKind::CpuOpcode
/// TokenKind::Label
/// TokenKind::Command
pub fn lex_identifier(c: CpuKind, text: &str) -> TokenKind {
    use CpuKind::*;
    use TokenKind::{Command, Label};
    match c {
        Cpu6809 => lex6809(text),
        Cpu6800 => lex6800(text),
        _ => panic!(),
    }
}

/// Classify an identifier after the raw Logos pass. Commands are target
/// independent; opcode lookup is delegated to the selected CPU backend.
pub fn classify_identifier(cpu: Option<CpuKind>, text: &str) -> TokenKind {
    if let Some(command) = command_kind(text) {
        return TokenKind::Command(*command);
    }

    match cpu {
        Some(cpu) => lex_identifier(cpu, text),
        None => TokenKind::Identifier,
    }
}

/// Returns either
/// TokenKind::CpuOpcode
/// TokenKind::Label
/// TokenKind::Command
pub fn get_identifier(input: TSpan) -> PResult<TokenKind> {
    // todo needs to handle local labels as well

    use CpuKind::*;

    use TokenKind::{Command, Identifier, Label};

    let c = input.extra().cpu_kind;

    let (rest, matched) = kind(Identifier)(input)?;

    let text = ascii_lowercase(get_str(&matched));
    let kind = classify_identifier(c, text.as_ref());

    match kind {
        TokenKind::Command(command) => Ok((rest, Command(command))),
        TokenKind::Identifier | TokenKind::CpuOpcode(_) | TokenKind::Label => Ok((rest, kind)),
        _ => err_nomatch(input),
    }
}
