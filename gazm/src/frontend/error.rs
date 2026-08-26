#![deny(unused_imports)]
use super::{to_pos, TSpan};
use crate::{help::ErrCode, vars::VarsErrorKind};
use grl_sources::{Position, SourceErrorType};
use grl_utils::FileError;
use thiserror::Error;
use unraveler::{ParseError, ParseErrorKind, Severity};

pub type PResult<'a, T> = Result<(TSpan<'a>, T), FrontEndError>;

impl From<ErrCode> for FrontEndErrorKind {
    fn from(value: ErrCode) -> Self {
        FrontEndErrorKind::HelpText(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum FrontEndErrorKind {
    #[error("{0}")]
    HelpText(ErrCode),
    #[error(transparent)]
    CpuAssembly(#[from] CpuAssemblyErrorKind),
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

    #[error("Unexpected character")]
    Unexpected,

    #[error("Invalid number")]
    InvalidNumber,

    #[error("Invalid string literal")]
    InvalidString,

    #[error("Invalid character literal")]
    InvalidCharacterLiteral,

    #[error("Expected close bracket ')'")]
    NoCloseBracket,
    #[error("Expected close square bracket ']'")]
    NoCloseSqBracket,
    #[error("Expected close brace '}}'")]
    NoCloseBrace,
}

pub type FeResult<T> = Result<T, FrontEndError>;

/// CPU-backend assembly errors, shared by all backends. Addressing-mode
/// payloads are stringified because each backend's mode type differs.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CpuAssemblyErrorKind {
    #[error("This {0} is not supported for this opcode")]
    ThisAddrModeUnsupported(String),
    #[error("Addressing mode is not supported for this opcode")]
    AddrModeUnsupported,
    #[error("This instruction only supports inherent mode addressing")]
    OnlySupports,
    #[error("Unknown {0} opcode")]
    UnknownOpcode(&'static str),
    #[error("This opcode needs an operand")]
    MissingOperand,
    #[error("Operands do not match any form of this opcode")]
    OperandsDontMatch,
}

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

impl<T> From<FrontEndError> for Result<T, FrontEndError> {
    fn from(value: FrontEndError) -> Self {
        Err(value)
    }
}

pub fn err_nomatch<T>(sp: TSpan) -> PResult<T> {
    Err(FrontEndError::error(sp, ParseErrorKind::NoMatch))
}

pub fn err_kind_nomatch(sp: TSpan) -> FrontEndError {
    FrontEndError::error(sp, ParseErrorKind::NoMatch)
}

pub fn err_error<T, E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E) -> PResult<T> {
    FrontEndError::error(sp, kind).into()
}

pub fn err_fatal<T, E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E) -> PResult<T> {
    FrontEndError::fatal(sp, kind).into()
}
pub fn error<E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E) -> FrontEndError {
    FrontEndError::error(sp, kind)
}

pub fn fatal<E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E) -> FrontEndError {
    FrontEndError::fatal(sp, kind)
}

impl FrontEndError {
    pub fn new<E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E, severity: Severity) -> Self {
        let position = to_pos(sp);
        Self {
            kind: kind.into(),
            position,
            severity,
        }
    }

    pub fn change_kind<E: Into<FrontEndErrorKind>>(self, k: E) -> Self {
        Self {
            kind: k.into(),
            ..self
        }
    }

    pub fn fatal<E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E) -> Self {
        let position = to_pos(sp);
        Self {
            kind: kind.into(),
            position,
            severity: Severity::Fatal,
        }
    }

    pub fn error_pos<E: Into<FrontEndErrorKind>>(position: Position, kind: E) -> Self {
        Self {
            kind: kind.into(),
            position,
            severity: Severity::Error,
        }
    }
    pub fn error<E: Into<FrontEndErrorKind>>(sp: TSpan, kind: E) -> Self {
        let position = to_pos(sp);
        Self {
            kind: kind.into(),
            position,
            severity: Severity::Error,
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

    fn merge(self, other: Self) -> Self {
        let severity = if self.severity == Severity::Fatal || other.severity == Severity::Fatal {
            Severity::Fatal
        } else {
            Severity::Error
        };

        if other.position.offset() > self.position.offset() {
            Self { severity, ..other }
        } else {
            Self { severity, ..self }
        }
    }

    fn append(input: TSpan, kind: ParseErrorKind, other: Self) -> Self {
        // Keep the diagnostic that got furthest through the input. If either
        // branch is fatal, preserve that severity while still preferring the
        // furthest position for useful editor diagnostics.
        let current = Self::from_error_kind(input, kind, Severity::Error);
        if current.position.offset() >= other.position.offset() {
            Self {
                severity: if current.severity == Severity::Fatal
                    || other.severity == Severity::Fatal
                {
                    Severity::Fatal
                } else {
                    current.severity
                },
                ..current
            }
        } else {
            Self {
                severity: if current.severity == Severity::Fatal
                    || other.severity == Severity::Fatal
                {
                    Severity::Fatal
                } else {
                    other.severity
                },
                ..other
            }
        }
    }
}

// TODO: Remove6809
