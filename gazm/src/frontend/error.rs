#![deny(unused_imports)]

use crate::vars::VarsErrorKind;

use super::{to_pos, TSpan};

use grl_sources::{grl_utils::FileError, Position, SourceErrorType};
use thiserror::Error;
use unraveler::{ParseError, ParseErrorKind, Severity};

pub type PResult<'a, T> = Result<(TSpan<'a>, T), FrontEndError>;

pub trait ErrorContext {
    fn context<T: Into<FrontEndErrorKind>>(self, e: T) -> Self;
}

impl<T> ErrorContext for PResult<'_, T> {
    fn context<K: Into<FrontEndErrorKind>>(self, kind: K) -> Self {
        self.map_err(|e| FrontEndError {
            kind: kind.into(),
            ..e
        })
    }
}

pub fn parse_fail<K: Into<FrontEndErrorKind>>(err: K, _sp: TSpan) -> super::FrontEndError {
    let pos = to_pos(_sp);
    FrontEndError {
        position: pos,
        kind: err.into(),
        severity: Severity::Fatal,
    }
}

pub fn parse_err<K: Into<FrontEndErrorKind>>(err: K, sp: TSpan) -> super::FrontEndError {
    FrontEndError {
        severity: Severity::Error,
        ..parse_fail(err, sp)
    }
}

#[derive(Debug, Error, Clone)]
pub enum AssemblyErrorKind {
    #[error("Post-increment indexing not valid indirectly")]
    PostIncNotValidIndirect,

    #[error("Pre-decrement indexing not valid indirectly")]
    PreDecNotValidIndirect,

    #[error("Expected a valid index register")]
    ExpectedValidIndexRegister,

    #[error("Expected a valid register")]
    ExpectedValidRegister,

    #[error("Duplicate registers in register set are not allowed")]
    InvalidRegisterSet,

    #[error("This addressing mode is not supported for this opcode")]
    AddrModeUnsupported,
}

impl Into<FrontEndErrorKind> for AssemblyErrorKind {
    fn into(self) -> FrontEndErrorKind {
        FrontEndErrorKind::AsmErrorKind(self)
    }
}

#[derive(Debug, Error, Clone)]
pub enum FrontEndErrorKind {
    #[error(transparent)]
    AsmErrorKind(AssemblyErrorKind),
    #[error(transparent)]
    SourceError(#[from] SourceErrorType),
    #[error(transparent)]
    FileError(#[from] FileError),
    #[error("vars error {0}")]
    VarsError(#[from] VarsErrorKind),
    #[error(transparent)]
    ParseError(#[from] ParseErrorKind),
    #[error("You cannot define a macro inside a macro definition")]
    IllegalMacroDefinition,
    #[error("Unable to find next line")]
    UnableToFindNextLine,

    #[error("Too many errors")]
    TooManyErrors(Vec<FrontEndError>),

    #[error("Unexpected character")]
    Unexpected,

    #[error("Expected close bracket ')'")]
    NoCloseBracket,
    #[error("Expected close square bracket ']'")]
    NoCloseSqBracket,
    #[error("Expected close brace '}}'")]
    NoCloseBrace,
}

pub type FeResult<T> = Result<T, FrontEndError>;

#[derive(Clone, Debug, Error)]
pub struct FrontEndError {
    pub position: Position,
    pub kind: FrontEndErrorKind,
    pub severity: Severity,
}

impl std::fmt::Display for FrontEndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        Ok(())
    }
}

impl FrontEndError {
    pub fn new(sp: TSpan, kind: FrontEndErrorKind, severity: Severity) -> Self {
        let position = to_pos(sp);
        Self {
            kind,
            position,
            severity,
        }
    }


    pub fn no_match_error(sp: TSpan) -> Self {
        Self::new(
            sp,
            FrontEndErrorKind::ParseError(ParseErrorKind::NoMatch),
            Severity::Fatal,
        )
    }
}

impl<'a> ParseError<TSpan<'a>> for FrontEndError {
    fn from_error_kind(input: TSpan, kind: ParseErrorKind, severity: Severity) -> Self {
        Self {
            position: to_pos(input),
            kind: FrontEndErrorKind::ParseError(kind),
            severity,
        }
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self {
        Self {
            kind: FrontEndErrorKind::ParseError(kind),
            ..self
        }
    }

    fn set_severity(self, severity: Severity) -> Self {
        Self { severity, ..self }
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn append(_input: TSpan, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}
