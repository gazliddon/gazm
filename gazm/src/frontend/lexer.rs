#![deny(unused_imports)]

use super::{basetoken::Token as BaseToken, ParseText};
use logos::{Lexer, Logos};
use std::ops::Range;

use crate::cpukind::CpuKind;

pub type Token<'a> = BaseToken<ParseText<'a>>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NumberKind {
    Hex,
    Dec,
    Bin,
    Char,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    InvalidCharacter,
    InvalidNumber,
    InvalidString,
    InvalidCharacterLiteral,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexError {
    pub span: Range<usize>,
    pub kind: LexErrorKind,
}

impl Default for LexError {
    fn default() -> Self {
        Self {
            span: 0..0,
            kind: LexErrorKind::InvalidCharacter,
        }
    }
}

impl LexError {
    fn at(span: Range<usize>, kind: LexErrorKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum CommandKind {
    Scope,
    GrabMem,
    Put,
    IncBin,
    IncBinRef,
    WriteBin,
    SetDp,
    ZeroFill,
    Fill,
    EmitWords,
    EmitString,
    EmitMessage,
    EmitBytes,
    EmitLongs,
    EmitQuads,
    ZeroWords,
    ReserveBytes,
    ReserveWords,
    ReserveLongs,
    Org,
    Include,
    Exec,
    Require,
    Import,
    Struct,
    Macro,
    Equ,
    Target,
    Section,
}

////////////////////////////////////////////////////////////////////////////////
fn parse_integer(txt: &str, radix: u8) -> Option<i64> {
    let mut value = 0i64;
    let mut found_digit = false;
    let mut previous_was_separator = false;
    for byte in txt.bytes() {
        if byte == b'_' {
            if !found_digit || previous_was_separator {
                return None;
            }
            previous_was_separator = true;
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        if digit >= radix {
            return None;
        }
        value = value.checked_mul(i64::from(radix))?;
        value = value.checked_add(i64::from(digit))?;
        found_digit = true;
        previous_was_separator = false;
    }
    (found_digit && !previous_was_separator).then_some(value)
}

fn from_bin(lex: &mut Lexer<TokenKind>) -> Result<(i64, NumberKind), LexError> {
    let text = lex.slice();
    let digits = text
        .strip_prefix('%')
        .or_else(|| text.get(2..))
        .ok_or_else(|| LexError::at(lex.span(), LexErrorKind::InvalidNumber))?;
    parse_integer(digits, 2)
        .map(|num| (num, NumberKind::Bin))
        .ok_or_else(|| LexError::at(lex.span(), LexErrorKind::InvalidNumber))
}

fn from_hex(lex: &mut Lexer<TokenKind>) -> Result<(i64, NumberKind), LexError> {
    let text = lex.slice();
    let digits = text
        .strip_prefix('$')
        .or_else(|| text.get(2..))
        .ok_or_else(|| LexError::at(lex.span(), LexErrorKind::InvalidNumber))?;
    parse_integer(digits, 16)
        .map(|num| (num, NumberKind::Hex))
        .ok_or_else(|| LexError::at(lex.span(), LexErrorKind::InvalidNumber))
}

fn from_char(lex: &mut Lexer<TokenKind>) -> Option<(i64, NumberKind)> {
    lex.slice()
        .chars()
        .nth(1)
        .map(|c| (c as i64, NumberKind::Char))
}

fn from_dec(lex: &mut Lexer<TokenKind>) -> Result<(i64, NumberKind), LexError> {
    parse_integer(lex.slice(), 10)
        .map(|num| (num, NumberKind::Dec))
        .ok_or_else(|| LexError::at(lex.span(), LexErrorKind::InvalidNumber))
}

fn from_float(lex: &mut Lexer<TokenKind>) -> Result<f64, LexError> {
    lex.slice()
        .parse::<f64>()
        .map_err(|_| LexError::at(lex.span(), LexErrorKind::InvalidNumber))
}

fn lex_error(lex: &mut Lexer<TokenKind>) -> LexError {
    let kind = match lex.slice().as_bytes().first() {
        Some(b'"') => LexErrorKind::InvalidString,
        Some(b'\'') => LexErrorKind::InvalidCharacterLiteral,
        _ => LexErrorKind::InvalidCharacter,
    };
    LexError::at(lex.span(), kind)
}

// `Float(f64)` makes TokenKind PartialEq-only (f64 is not Eq).
#[derive(Logos, Copy, Clone, Debug, PartialEq)]
#[logos(error(LexError, callback = lex_error))]
#[logos(skip r"[ \t\f\n]+")]
// Identifiers: a letter/underscore start, then word chars with single
// dots allowed between them (`proc.data`). A run of dots can never be an
// identifier, so `..` stays free as the range operator; labels starting
// with a dot (.COAST, .OPTR, .IF) get their own rule below.
#[logos(subpattern id_al = r"[a-zA-Z_]")]
#[logos(subpattern id_body = r"[a-zA-Z0-9_]")]
#[logos(subpattern id = r"(?&id_al)(?:(?&id_body)|\.(?&id_body))*")]
#[logos(subpattern pre_hex = r"(0[xX]|\$)")]
#[logos(subpattern pre_bin = r"(0[bB]|%)")]
pub enum TokenKind {
    Error,
    // Assigned by the CPU-specific parser after the Logos pass. Keeping this
    // distinct from Identifier makes the parser's classification explicit.
    CpuOpcode(CpuKind),
    Command(CommandKind),
    Label,

    #[regex("!(?&id)")]
    LocalIdentifier,

    // #[regex(r"\[\[[^\]]*\]\]", priority=10)]
    #[regex(r"```[^`]*```", priority = 10)]
    BigDocText,

    // Identifiers are dot-free so `..` can be a range operator; labels
    // starting with a dot (.COAST, .OPTR, .IF) get their own rule.
    #[regex("(?&id)")]
    #[regex(r"\.[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Range operator for `for i in 0..N`.
    #[token("..")]
    DoubleDot,

    // Consume identifier-like suffixes as part of a number so inputs such as
    // `123abc` and `$12G` produce one useful invalid-number diagnostic rather
    // than silently becoming two unrelated tokens.
    #[regex(r"[0-9][0-9a-zA-Z_]*", from_dec)]
    #[regex(r"(?&pre_hex)[0-9a-zA-Z_]*", from_hex, priority = 3)]
    #[regex(r"(?&pre_bin)[0-9a-zA-Z_]*", from_bin, priority = 3)]
    #[regex(r"'.'", from_char)]
    Number((i64, NumberKind)),

    // Float literal. The `..` range operator can't be confused with it:
    // a dot is only part of a float when digits follow it, and `0..256`
    // has a dot followed by a dot. Floats exist at assembly time only;
    // emitting one requires an explicit conversion like `round()`.
    #[regex(r"[0-9]+\.[0-9]+", from_float, priority = 4)]
    Float(f64),

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    CloseSquareBracket,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("'")]
    Apostrophe,

    #[token("(")]
    OpenBracket,

    #[token(")")]
    CloseBracket,

    #[token("*")]
    Star,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,

    // #[token("\\")]
    // BackSlash,
    #[regex(r";;;.*?(\n|$)")]
    DocComment,

    #[regex(r"(;|//).*?(\n|$)")]
    Comment,

    #[token("&")]
    Ampersand,

    // Scoped names: `::core::SLEEP` (imports, absolute) or
    // `proc::time` (struct fields, relative) — one or more `::` segments.
    #[regex(r"(?:::(?&id)|(?&id))(::(?&id))+")]
    FqnIdentifier,

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    QuotedString,

    #[token(",")]
    Comma,

    #[token("::")]
    DoubleColon,

    #[token(":")]
    Colon,

    #[token(">>")]
    DoubleGreaterThan,

    #[token("<<")]
    DoubleLessThan,

    #[token(">=")]
    GreaterThanEqual,

    #[token("<=")]
    LessThanEqual,

    #[token(">")]
    GreaterThan,

    #[token("<")]
    LessThan,

    #[token("==")]
    DoubleEquals,

    #[token("!=")]
    NotEquals,

    #[token("&&")]
    DoubleAmpersand,

    #[token("||")]
    DoubleBar,

    #[token("|")]
    Bar,

    #[token("^")]
    Caret,

    #[token("#")]
    Hash,

    #[token("!")]
    Pling,

    #[token("@")]
    At,

    #[token("=")]
    Equals,
}

pub fn to_tokens_no_comment(source_file: &grl_sources::SourceFile) -> Vec<Token<'_>> {
    use TokenKind::*;
    let not_comment = |k: &TokenKind| k != &DocComment && k != &Comment;
    let tokens = to_tokens_filter(source_file, not_comment);
    tokens
}

/// Tokenize a source file while retaining lexer failures and their source
/// ranges. Invalid lexemes are omitted from the parser stream; callers should
/// report the returned errors to the user.
pub fn to_tokens_no_comment_with_errors(
    source_file: &grl_sources::SourceFile,
) -> (Vec<Token<'_>>, Vec<LexError>) {
    use TokenKind::*;
    let not_comment = |k: &TokenKind| k != &DocComment && k != &Comment && k != &Error;
    let (ret, errors) = to_tokens_kinds(source_file);
    let tokens = ret
        .into_iter()
        .filter(|(tk, _)| not_comment(tk))
        .map(|(tk, r)| Token::new(tk, ParseText::new(source_file, r)))
        .collect();
    (tokens, errors)
}

fn to_tokens_kinds(
    source_file: &grl_sources::SourceFile,
) -> (Vec<(TokenKind, std::ops::Range<usize>)>, Vec<LexError>) {
    let mut errors = Vec::new();
    let tokens = TokenKind::lexer(&source_file.get_text().source)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => (kind, pos),
            Err(error) => {
                errors.push(error);
                (TokenKind::Error, pos)
            }
        })
        .collect();
    (tokens, errors)
}

/// Converts a source file into a vector of tokens.
///
/// # Arguments
/// * `source_file`: The source file to convert.
/// * `predicate`: A predicate that determines which token kinds to include in the output.
///
/// # Returns
/// A vector of tokens, where each token is created by calling the `Token::new` constructor with the given token kind and parse text.
///
/// # Examples
/// ```
/// use gazm::frontend::{create_source_file, to_tokens_filter, TokenKind};
///
/// let source_file = create_source_file("Hello, world!");
/// let tokens = to_tokens_filter(&source_file, |tk| *tk == TokenKind::Identifier);
/// assert_eq!(tokens.len(), 2);
/// ```
pub fn to_tokens_filter<P>(source_file: &grl_sources::SourceFile, predicate: P) -> Vec<Token<'_>>
where
    P: Fn(&TokenKind) -> bool,
{
    let (ret, _) = to_tokens_kinds(source_file);

    ret.into_iter()
        .filter(|(tk, _)| predicate(tk))
        .map(|(tk, r)| {
            let pt = ParseText::new(source_file, r);
            let t: Token = Token::new(tk, pt);
            t
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_line_comment_is_tokenized() {
        let source = crate::frontend::create_source_file("; final comment");
        let tokens = to_tokens_filter(&source, |_| true);
        assert_eq!(
            tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
            vec![TokenKind::Comment]
        );
    }

    #[test]
    fn overflowing_number_becomes_error_token() {
        let source = crate::frontend::create_source_file("999999999999999999999999999999");
        let tokens = to_tokens_filter(&source, |_| true);
        assert_eq!(
            tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
            vec![TokenKind::Error]
        );
    }

    #[test]
    fn numeric_separators_must_be_between_digits() {
        for text in ["1_", "1__2", "$__ff", "%101_"] {
            let source = crate::frontend::create_source_file(text);
            let (_, errors) = to_tokens_no_comment_with_errors(&source);
            assert_eq!(errors.len(), 1, "expected one lexer error for {text:?}");
            assert_eq!(errors[0].kind, LexErrorKind::InvalidNumber);
            assert_eq!(errors[0].span, 0..text.len());
        }
    }

    #[test]
    fn numeric_suffixes_are_not_silently_split_into_tokens() {
        for text in ["123abc", "$12G", "%102", "0x12Q"] {
            let source = crate::frontend::create_source_file(text);
            let (_, errors) = to_tokens_no_comment_with_errors(&source);
            assert_eq!(errors.len(), 1, "expected one lexer error for {text:?}");
            assert_eq!(errors[0].kind, LexErrorKind::InvalidNumber);
            assert_eq!(errors[0].span, 0..text.len());
        }
    }

    #[test]
    fn valid_numeric_separators_still_tokenize() {
        for (text, expected) in [
            ("1_000", NumberKind::Dec),
            ("$ab_cd", NumberKind::Hex),
            ("%1010_0011", NumberKind::Bin),
        ] {
            let source = crate::frontend::create_source_file(text);
            let (tokens, errors) = to_tokens_no_comment_with_errors(&source);
            assert!(errors.is_empty(), "unexpected lexer error for {text:?}");
            assert!(matches!(tokens[0].kind, TokenKind::Number((_, kind)) if kind == expected));
        }
    }

    #[test]
    fn float_literals_tokenize_as_floats() {
        for text in ["3.14", "0.5", "127.0"] {
            let source = crate::frontend::create_source_file(text);
            let (tokens, errors) = to_tokens_no_comment_with_errors(&source);
            assert!(errors.is_empty(), "unexpected lexer error for {text:?}");
            assert!(
                matches!(tokens[0].kind, TokenKind::Float(..)),
                "{text:?} -> {:?}",
                tokens[0].kind
            );
        }
    }

    #[test]
    fn range_operator_is_not_a_float() {
        // `0..256` must stay Number(0), DoubleDot, Number(256) — the float
        // rule requires digits after the dot.
        let source = crate::frontend::create_source_file("0..256");
        let (tokens, errors) = to_tokens_no_comment_with_errors(&source);
        assert!(errors.is_empty(), "unexpected lexer error");
        assert_eq!(tokens.len(), 3, "{tokens:?}");
        assert!(matches!(tokens[0].kind, TokenKind::Number((0, _))));
        assert!(matches!(tokens[1].kind, TokenKind::DoubleDot));
        assert!(matches!(tokens[2].kind, TokenKind::Number((256, _))));
    }

    #[test]
    fn malformed_literals_have_specific_errors() {
        let source = crate::frontend::create_source_file("\"unterminated");
        let (_, errors) = to_tokens_no_comment_with_errors(&source);
        assert_eq!(errors[0].kind, LexErrorKind::InvalidString);
    }

    #[test]
    fn apostrophe_is_a_token_not_a_char_literal_error() {
        // The Z80 backend needs `'` (EX AF,AF'); a run like `'ab'` is now
        // three valid tokens rather than a malformed character literal.
        let source = crate::frontend::create_source_file("'ab'");
        let (tokens, errors) = to_tokens_no_comment_with_errors(&source);
        assert!(errors.is_empty(), "unexpected lexer errors: {errors:?}");
        assert_eq!(
            tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
            vec![
                TokenKind::Apostrophe,
                TokenKind::Identifier,
                TokenKind::Apostrophe,
            ]
        );
    }

    #[test]
    fn square_brackets_tokenize_as_indexing_delimiters() {
        let source_file = crate::frontend::create_source_file("ldx [CRPROC]");
        let (tokens, errors) = to_tokens_no_comment_with_errors(&source_file);

        assert!(errors.is_empty(), "unexpected lexer errors: {errors:?}");
        assert_eq!(
            tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
            vec![
                TokenKind::Identifier,
                TokenKind::OpenSquareBracket,
                TokenKind::Identifier,
                TokenKind::CloseSquareBracket,
            ]
        );
    }
}
