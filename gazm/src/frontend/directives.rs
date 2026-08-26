//! Per-CPU directive vocabularies.
//!
//! Directive *semantics* are CPU-independent: every directive parses to a
//! shared [`AstNodeKind`] (`Fcb`, `Fdb`, `Rmb`, ...) that the sizer and
//! compiler handle generically. Only the *names* differ per CPU — `fcb` on
//! the 6800/6809, `db`/`.byte` on the 6502, and so on. Each CPU's
//! vocabulary is a data table here; both the classification path
//! ([`command_kind`]) and the parse path ([`CommandKind`]'s `Parser` impl)
//! consult it, so an alias behaves exactly like its canonical spelling.
//!
//! Names are lowercase; lookups are case-insensitive. Dot-prefixed names
//! (`.byte`) work because the lexer lexes `.word`-shaped identifiers as
//! plain identifiers.

use crate::cpukind::CpuKind;

use super::CommandKind;

/// The shared 6800/6809 directive vocabulary. Everything except `SetDp`,
/// which is 6809-only: the 6800 has no direct-page register, and 6809
/// sources such as Stargate's `SETDP RAM>>8` rely on it.
pub const BASE_DIRECTIVES: &[(&str, CommandKind)] = &[
    ("scope", CommandKind::Scope),
    ("grabmem", CommandKind::GrabMem),
    ("put", CommandKind::Put),
    ("incbin", CommandKind::IncBin),
    ("incbinref", CommandKind::IncBinRef),
    ("writebin", CommandKind::WriteBin),
    ("bsz", CommandKind::ZeroFill),
    ("fill", CommandKind::Fill),
    ("fdb", CommandKind::EmitWords),
    ("fcc", CommandKind::EmitString),
    ("fcb", CommandKind::EmitBytes),
    ("zmb", CommandKind::ZeroFill),
    ("zmd", CommandKind::ZeroWords),
    ("rmb", CommandKind::ReserveBytes),
    ("rzb", CommandKind::ZeroFill),
    ("org", CommandKind::Org),
    ("include", CommandKind::Include),
    ("exec", CommandKind::Exec),
    ("require", CommandKind::Require),
    ("import", CommandKind::Import),
    ("struct", CommandKind::Struct),
    ("macro", CommandKind::Macro),
    ("equ", CommandKind::Equ),
    ("target", CommandKind::Target),
    ("section", CommandKind::Section),
];

/// The 6809 vocabulary: the shared set plus `setdp`.
pub const DIRECTIVES_6809: &[(&str, CommandKind)] = &[
    ("scope", CommandKind::Scope),
    ("grabmem", CommandKind::GrabMem),
    ("put", CommandKind::Put),
    ("incbin", CommandKind::IncBin),
    ("incbinref", CommandKind::IncBinRef),
    ("writebin", CommandKind::WriteBin),
    ("bsz", CommandKind::ZeroFill),
    ("fill", CommandKind::Fill),
    ("fdb", CommandKind::EmitWords),
    ("fcc", CommandKind::EmitString),
    ("fcb", CommandKind::EmitBytes),
    ("zmb", CommandKind::ZeroFill),
    ("zmd", CommandKind::ZeroWords),
    ("rmb", CommandKind::ReserveBytes),
    ("rzb", CommandKind::ZeroFill),
    ("org", CommandKind::Org),
    ("include", CommandKind::Include),
    ("exec", CommandKind::Exec),
    ("require", CommandKind::Require),
    ("import", CommandKind::Import),
    ("struct", CommandKind::Struct),
    ("macro", CommandKind::Macro),
    ("equ", CommandKind::Equ),
    ("target", CommandKind::Target),
    ("section", CommandKind::Section),
    ("setdp", CommandKind::SetDp),
];

/// The 6502/65C02 vocabulary: classic 6502 directive names (`db`, `dw`,
/// `ds`) plus dot-suffixed aliases (`.byte`, `.word`, `.res`). The 6800/9
/// spellings (`fdb`, `fcb`) are intentionally absent.
pub const DIRECTIVES_6502: &[(&str, CommandKind)] = &[
    ("scope", CommandKind::Scope),
    ("grabmem", CommandKind::GrabMem),
    ("put", CommandKind::Put),
    ("incbin", CommandKind::IncBin),
    ("incbinref", CommandKind::IncBinRef),
    ("writebin", CommandKind::WriteBin),
    ("bsz", CommandKind::ZeroFill),
    ("fill", CommandKind::Fill),
    ("fcc", CommandKind::EmitString),
    ("zmb", CommandKind::ZeroFill),
    ("zmd", CommandKind::ZeroWords),
    ("rmb", CommandKind::ReserveBytes),
    ("rzb", CommandKind::ZeroFill),
    ("org", CommandKind::Org),
    ("include", CommandKind::Include),
    ("exec", CommandKind::Exec),
    ("require", CommandKind::Require),
    ("import", CommandKind::Import),
    ("struct", CommandKind::Struct),
    ("macro", CommandKind::Macro),
    ("equ", CommandKind::Equ),
    ("target", CommandKind::Target),
    ("section", CommandKind::Section),
    ("db", CommandKind::EmitBytes),
    (".byte", CommandKind::EmitBytes),
    ("dw", CommandKind::EmitWords),
    (".word", CommandKind::EmitWords),
    ("ds", CommandKind::ReserveBytes),
    (".res", CommandKind::ReserveBytes),
];

/// The 68000 vocabulary: `dc.b/w/l` emit and `ds.b/w/l` reserve at byte,
/// word, and long widths. The 6800/9 spellings (`fcb`, `fdb`, `rmb`) are
/// intentionally absent.
pub const DIRECTIVES_68000: &[(&str, CommandKind)] = &[
    ("scope", CommandKind::Scope),
    ("grabmem", CommandKind::GrabMem),
    ("put", CommandKind::Put),
    ("incbin", CommandKind::IncBin),
    ("incbinref", CommandKind::IncBinRef),
    ("writebin", CommandKind::WriteBin),
    ("fill", CommandKind::Fill),
    ("org", CommandKind::Org),
    ("include", CommandKind::Include),
    ("exec", CommandKind::Exec),
    ("require", CommandKind::Require),
    ("import", CommandKind::Import),
    ("struct", CommandKind::Struct),
    ("macro", CommandKind::Macro),
    ("equ", CommandKind::Equ),
    ("target", CommandKind::Target),
    ("section", CommandKind::Section),
    ("dc.b", CommandKind::EmitBytes),
    ("dc.w", CommandKind::EmitWords),
    ("dc.l", CommandKind::EmitLongs),
    ("ds.b", CommandKind::ReserveBytes),
    ("ds.w", CommandKind::ReserveWords),
    ("ds.l", CommandKind::ReserveLongs),
];

/// The Z80 vocabulary: the classic `defb`/`defw`/`defs`/`defm` spellings
/// (SkoolKit, z80asm) plus the shorter `db`/`dw`/`ds`.
pub const DIRECTIVES_Z80: &[(&str, CommandKind)] = &[
    ("scope", CommandKind::Scope),
    ("grabmem", CommandKind::GrabMem),
    ("put", CommandKind::Put),
    ("incbin", CommandKind::IncBin),
    ("incbinref", CommandKind::IncBinRef),
    ("writebin", CommandKind::WriteBin),
    ("fill", CommandKind::Fill),
    ("org", CommandKind::Org),
    ("include", CommandKind::Include),
    ("exec", CommandKind::Exec),
    ("require", CommandKind::Require),
    ("import", CommandKind::Import),
    ("struct", CommandKind::Struct),
    ("macro", CommandKind::Macro),
    ("equ", CommandKind::Equ),
    ("target", CommandKind::Target),
    ("section", CommandKind::Section),
    ("defb", CommandKind::EmitBytes),
    ("defw", CommandKind::EmitWords),
    ("defs", CommandKind::ReserveBytes),
    ("defm", CommandKind::EmitString),
    ("db", CommandKind::EmitBytes),
    ("dw", CommandKind::EmitWords),
    ("ds", CommandKind::ReserveBytes),
];

/// The full directive vocabulary for a CPU: canonical names plus aliases.
/// Unimplemented CPUs inherit the shared table as a placeholder.
pub fn directives_for(cpu: CpuKind) -> &'static [(&'static str, CommandKind)] {
    match cpu {
        CpuKind::Cpu6809 => DIRECTIVES_6809,
        CpuKind::Cpu6800 => BASE_DIRECTIVES,
        CpuKind::Cpu6502 | CpuKind::Cpu65c02 => DIRECTIVES_6502,
        CpuKind::CpuZ80 => DIRECTIVES_Z80,
        CpuKind::Cpu68000 => DIRECTIVES_68000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::{classify_identifier, TokenKind};

    #[test]
    fn setdp_is_6809_only() {
        assert_eq!(
            classify_identifier(Some(CpuKind::Cpu6809), "setdp"),
            TokenKind::Command(CommandKind::SetDp)
        );
        // Case-insensitive, as Stargate's `SETDP RAM>>8` uses.
        assert_eq!(
            classify_identifier(Some(CpuKind::Cpu6809), "SETDP"),
            TokenKind::Command(CommandKind::SetDp)
        );
        // Not a command on the 6800: it falls through to label.
        assert_eq!(
            classify_identifier(Some(CpuKind::Cpu6800), "setdp"),
            TokenKind::Label
        );
    }

    #[test]
    fn aliases_classify_per_cpu() {
        for (name, kind) in [
            ("db", CommandKind::EmitBytes),
            (".byte", CommandKind::EmitBytes),
            ("dw", CommandKind::EmitWords),
            (".word", CommandKind::EmitWords),
            ("ds", CommandKind::ReserveBytes),
            (".res", CommandKind::ReserveBytes),
        ] {
            assert_eq!(
                classify_identifier(Some(CpuKind::Cpu6502), name),
                TokenKind::Command(kind),
                "{name} should classify on 6502"
            );
            // The same words are plain labels on the 6809.
            assert_eq!(
                classify_identifier(Some(CpuKind::Cpu6809), name),
                TokenKind::Label,
                "{name} should not classify on 6809"
            );
        }
    }

    #[test]
    fn canonical_names_are_cpu_scoped() {
        assert_eq!(
            classify_identifier(Some(CpuKind::Cpu6809), "fcb"),
            TokenKind::Command(CommandKind::EmitBytes)
        );
        // 6800/9 spellings are not 6502 directives.
        assert_eq!(
            classify_identifier(Some(CpuKind::Cpu6502), "fdb"),
            TokenKind::Label
        );
        // Shared names classify everywhere.
        assert_eq!(
            classify_identifier(Some(CpuKind::Cpu6502), "org"),
            TokenKind::Command(CommandKind::Org)
        );
    }

    #[test]
    fn m68k_directives_classify() {
        for (name, kind) in [
            ("dc.b", CommandKind::EmitBytes),
            ("dc.w", CommandKind::EmitWords),
            ("dc.l", CommandKind::EmitLongs),
            ("ds.b", CommandKind::ReserveBytes),
            ("ds.w", CommandKind::ReserveWords),
            ("ds.l", CommandKind::ReserveLongs),
        ] {
            assert_eq!(
                classify_identifier(Some(CpuKind::Cpu68000), name),
                TokenKind::Command(kind),
                "{name} should classify on 68000"
            );
            // The same words are plain labels on the 6809.
            assert_eq!(
                classify_identifier(Some(CpuKind::Cpu6809), name),
                TokenKind::Label,
                "{name} should not classify on 6809"
            );
        }
    }
}
