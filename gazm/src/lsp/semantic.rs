//! Per-target semantic tokens for the LSP.
//!
//! The tree-sitter grammar is CPU-agnostic (one union grammar for all
//! targets), so it cannot know which words are mnemonics, directives, or
//! registers on a given CPU. The LSP, by contrast, knows the document's
//! target (and therefore its CPU), so it re-classifies each identifier
//! with the target's vocabulary and emits semantic tokens the client
//! renders on top of (or instead of) the grammar captures.

use crate::cpukind::CpuKind;
use crate::frontend::{classify_identifier, to_tokens_filter, TokenKind};
use grl_sources::{AsmSource, SourceFile};
use serde_json::{json, Value};

/// The token types in the server's legend, in order. Clients map these to
/// highlight groups (`@lsp.type.keyword`, `@lsp.type.function`, ...).
pub const TOKEN_TYPES: &[&str] = &[
    "keyword", "function", "variable", "number", "string", "comment",
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TokenType {
    Keyword,
    Function,
    Variable,
    Number,
    String,
    Comment,
}

impl TokenType {
    fn index(self) -> usize {
        match self {
            TokenType::Keyword => 0,
            TokenType::Function => 1,
            TokenType::Variable => 2,
            TokenType::Number => 3,
            TokenType::String => 4,
            TokenType::Comment => 5,
        }
    }
}

/// Compute the `data` payload of a `textDocument/semanticTokens/full`
/// response for one document, classified with the target's CPU.
pub fn semantic_tokens_full(text: &str, cpu: CpuKind) -> Value {
    let source = SourceFile::new("<lsp>", text, AsmSource::FromStr);
    let tokens = to_tokens_filter(&source, |kind| *kind != TokenKind::Error);

    let mut data: Vec<u64> = Vec::new();
    let mut prev_line = 0usize;
    let mut prev_char = 0usize;

    for token in &tokens {
        let Some(tt) = classify_token(&token.kind, &token.extra.get_text(), cpu) else {
            continue;
        };
        let (line, char) = token.extra.pos.line_col();
        let len = token.extra.pos.range().len();
        if len == 0 {
            continue;
        }
        // Delta encoding: each token is relative to the previous one.
        let delta_line = line - prev_line;
        let delta_char = if delta_line == 0 {
            char - prev_char
        } else {
            char
        };
        data.push(delta_line as u64);
        data.push(delta_char as u64);
        data.push(len as u64);
        data.push(tt.index() as u64);
        data.push(0); // modifiers
        prev_line = line;
        prev_char = char;
    }

    json!({ "data": data })
}

/// Map a lexer token to a semantic-token type using the target CPU's
/// vocabulary. Words that are neither keywords nor opcodes nor registers
/// (plain labels, unknown identifiers) are skipped — the grammar already
/// highlights them.
fn classify_token(kind: &TokenKind, text: &str, cpu: CpuKind) -> Option<TokenType> {
    use TokenKind::*;
    match kind {
        Comment | DocComment => Some(TokenType::Comment),
        QuotedString | SingleQuotedString => Some(TokenType::String),
        Number(_) | Float(_) => Some(TokenType::Number),
        Identifier | Label | CpuOpcode(_) | Command(_) => {
            let classified = classify_identifier(Some(cpu), text);
            match classified {
                Command(_) => Some(TokenType::Keyword),
                CpuOpcode(_) => Some(TokenType::Function),
                _ => {
                    if is_register(cpu, text) {
                        Some(TokenType::Variable)
                    } else {
                        None
                    }
                }
            }
        }
        _ => None,
    }
}

/// Is `text` a register name in this CPU's vocabulary?
fn is_register(cpu: CpuKind, text: &str) -> bool {
    let t = text.to_ascii_lowercase();
    let reg_num = |prefix: char, max: u8| -> bool {
        t.len() == 2 && t.starts_with(prefix) && t[1..].parse::<u8>().map_or(false, |n| n < max)
    };
    match cpu {
        CpuKind::Cpu68000 => {
            matches!(t.as_str(), "sp" | "pc" | "sr" | "ccr" | "usp")
                || reg_num('d', 8)
                || reg_num('a', 8)
        }
        CpuKind::CpuJagrisc => t.starts_with('r') && t[1..].parse::<u8>().map_or(false, |n| n < 32),
        CpuKind::Cpu6809 => matches!(
            t.as_str(),
            "a" | "b" | "d" | "dp" | "x" | "y" | "u" | "s" | "pc" | "pcr" | "cc"
        ),
        CpuKind::Cpu6800 => matches!(t.as_str(), "a" | "b" | "cc" | "sp" | "x" | "pc"),
        CpuKind::Cpu6502 | CpuKind::Cpu65c02 => {
            matches!(t.as_str(), "a" | "x" | "y" | "sp" | "pc" | "p")
        }
        CpuKind::CpuZ80 => matches!(
            t.as_str(),
            "a" | "b"
                | "c"
                | "d"
                | "e"
                | "h"
                | "l"
                | "af"
                | "bc"
                | "de"
                | "hl"
                | "ix"
                | "iy"
                | "sp"
                | "pc"
                | "i"
                | "r"
        ),
    }
}
