//! The CPU backend registry.
//!
//! Each backend registers its dispatch entry points here; the shared
//! assembler and frontend generate their per-CPU matches from this single
//! macro, so adding a CPU touches exactly one place (plus the backend's own
//! modules). The macros are expanded with `#[macro_use]` crate-wide.
//!
//! `cpu_dispatch!` expands to a complete `match` for one of the dispatch
//! sites:
//! - `compile` / `size`: `Assembler::compile_node` / `size_node`
//!   (dispatched on `CpuSpecific`).
//! - `parse_multi`: the frontend statement parser (dispatched on
//!   `CpuKind`).
//! - `lex`: the identifier classifier (dispatched on `CpuKind`).

macro_rules! cpu_dispatch {
    (compile, $self:ident, $node:ident, $node_kind:ident, $scope:ident) => {
        match $node_kind {
            crate::frontend::CpuSpecific::Cpu6809(nk) => {
                crate::cpu6809::assembler::compile_node($self, $node, nk, $scope)
            }
            crate::frontend::CpuSpecific::Cpu6800(nk) => {
                crate::cpu6800::assembler::compile_node($self, $node, nk, $scope)
            }
            crate::frontend::CpuSpecific::CpuZ80(nk) => {
                crate::cpu_z80::assembler::compile_node($self, $node, nk, $scope)
            }
        }
    };
    (size, $self:ident, $sizer:ident, $id:ident, $node_kind:ident) => {
        match $node_kind {
            crate::frontend::CpuSpecific::Cpu6809(nk) => {
                crate::cpu6809::assembler::size_node_internal($sizer, $self, $id, nk)
            }
            crate::frontend::CpuSpecific::Cpu6800(nk) => {
                crate::cpu6800::assembler::size_node_internal($sizer, $self, $id, nk)
            }
            crate::frontend::CpuSpecific::CpuZ80(nk) => {
                crate::cpu_z80::assembler::size_node_internal($sizer, $self, $id, nk)
            }
        }
    };
    (parse_multi, $cpu:ident, $input:ident) => {
        match $cpu {
            crate::cpukind::CpuKind::Cpu6809 => {
                crate::cpu6809::frontend::parse_multi_opcode_vec($input)
            }
            crate::cpukind::CpuKind::Cpu6800 => {
                crate::cpu6800::frontend::parse_multi_opcode_vec($input)
            }
            crate::cpukind::CpuKind::CpuZ80 => {
                crate::cpu_z80::frontend::parse_multi_opcode_vec($input)
            }
            _ => {
                return Err(crate::frontend::error::FrontEndError::error(
                    $input,
                    crate::frontend::FrontEndErrorKind::Unexpected,
                ))
            }
        }
    };
    (lex, $cpu:ident, $text:ident) => {
        match $cpu {
            crate::cpukind::CpuKind::Cpu6809 => crate::cpu6809::frontend::lex_identifier($text),
            crate::cpukind::CpuKind::Cpu6800 => crate::cpu6800::frontend::lex_identifier($text),
            crate::cpukind::CpuKind::CpuZ80 => crate::cpu_z80::frontend::lex_identifier($text),
            // Unimplemented backends: no opcodes are recognized yet, so any
            // word that is not a directive classifies as a label.
            _ => crate::frontend::TokenKind::Label,
        }
    };
}
