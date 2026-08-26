use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use unraveler::{kind, Collection};

use crate::{
    cpu6800::frontend::lex_identifier as lex6800,
    cpu6809::frontend::lex_identifier as lex6809,
    cpukind::CpuKind,
    frontend::{directives::directives_for, err_nomatch, get_str, get_text},
};

use super::{CommandKind, PResult, TSpan, TokenKind};

/// Per-CPU assembler directive vocabularies, indexed by
/// `(cpu, lowercase name)`. Built once from [`directives_for`]: each CPU's
/// table maps its directive spellings (`db`, `.byte`, ...) to the shared
/// semantic [`CommandKind`]s.
pub static DIRECTIVES: LazyLock<HashMap<CpuKind, HashMap<String, CommandKind>>> =
    LazyLock::new(|| {
        let mut by_cpu: HashMap<CpuKind, HashMap<String, CommandKind>> = HashMap::new();
        for cpu in [
            CpuKind::Cpu6809,
            CpuKind::Cpu6800,
            CpuKind::Cpu6502,
            CpuKind::Cpu65c02,
            CpuKind::CpuZ80,
            CpuKind::Cpu68000,
        ] {
            by_cpu.insert(
                cpu,
                directives_for(cpu)
                    .iter()
                    .map(|(name, kind)| (name.to_string(), *kind))
                    .collect(),
            );
        }
        by_cpu
    });

/// Lowercase ASCII text without allocating when it is already lowercase.
pub fn ascii_lowercase(text: &str) -> Cow<'_, str> {
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(text.to_ascii_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

pub fn command_kind(cpu: CpuKind, text: &str) -> Option<CommandKind> {
    let lowered = ascii_lowercase(text);
    DIRECTIVES
        .get(&cpu)
        .and_then(|table| table.get(lowered.as_ref()))
        .copied()
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
    use TokenKind::Label;
    match c {
        Cpu6809 => lex6809(text),
        Cpu6800 => lex6800(text),
        CpuZ80 => crate::cpu_z80::frontend::lex_identifier(text),
        // Unimplemented backends: no opcodes are recognized yet, so any
        // word that is not a directive classifies as a label.
        _ => Label,
    }
}

/// Classify an identifier after the raw Logos pass. Directives come from
/// the selected CPU's vocabulary table; opcode lookup is delegated to the
/// selected CPU backend.
pub fn classify_identifier(cpu: Option<CpuKind>, text: &str) -> TokenKind {
    if let Some(command) = command_kind(cpu.unwrap_or_default(), text) {
        return TokenKind::Command(command);
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
