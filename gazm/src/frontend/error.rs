#![deny(unused_imports)]
use super::item6809::AddrModeParseType;
use crate::error::{ErrorMessage, UserError, UserErrorData};
use crate::vars::VarsErrorKind;

use super::{to_pos, TSpan};

use crate::help::ErrCode;
use grl_sources::{grl_utils::FileError, Position, SourceErrorType, SourceFile, TextEditTrait};
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

#[derive(Debug, Error, Clone, PartialEq, Copy)]
pub enum AssemblyErrorKind {
    #[error("This {0:?} is not supported for this opcode")]
    ThisAddrModeUnsupported(AddrModeParseType),

    #[error("Addressing mode is not supported for this opcode")]
    AddrModeUnsupported,

    #[error("This instruction only supports inherent mode addressing")]
    OnlySupports(AddrModeParseType),
}

impl Into<FrontEndErrorKind> for ErrCode {
    fn into(self) -> FrontEndErrorKind {
        FrontEndErrorKind::HelpText(self)
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum FrontEndErrorKind {
    #[error("{0}")]
    HelpText(ErrCode),
    #[error(transparent)]
    AsmErrorKind(#[from] AssemblyErrorKind),
    #[error(transparent)]
    SourceError(#[from] SourceErrorType),
    #[error(transparent)]
    FileError(#[from] FileError),
    #[error("vars error {0}")]
    VarsError(#[from] VarsErrorKind),

    #[error(transparent)]
    ParseError(#[from] ParseErrorKind),

    #[error("Too many errors")]
    TooManyErrors,

    #[error("You cannot define a macro inside a macro definition")]
    IllegalMacroDefinition,
    #[error("Unable to find next line")]
    UnableToFindNextLine,

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

impl<T> Into<Result<T, FrontEndError>> for FrontEndError {
    fn into(self) -> Result<T, Self> {
        Err(self)
    }
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

    fn append(_input: TSpan, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}

fn get_line(sf: &SourceFile, line: isize) -> String {
    if line < 0 {
        String::new()
    } else {
        let txt = sf.get_text().get_line(line as usize).unwrap_or("");
        format!("{txt}")
    }
}

fn get_lines(sf: &SourceFile, line: isize) -> String {
    [
        get_line(sf, line - 2),
        get_line(sf, line - 1),
        get_line(sf, line),
        get_line(sf, line + 1),
        get_line(sf, line + 2),
    ]
    .join("\n")
}

pub fn to_user_error(e: FrontEndError, sf: &SourceFile) -> UserError {
    use ErrorMessage::*;

    let message = match e.kind {
        FrontEndErrorKind::HelpText(ht) => {
            let short = crate::help::HELP.get_short(ht);
            let full_text = crate::help::HELP.get(ht);
            Markdown(format!("{short}"), format!("{full_text}"))
        }
        _ => Plain(format!("{e}")),
    };

    let line = e.position.line();
    let line = get_line(sf, line as isize);

    let ued = UserErrorData {
        message,
        pos: e.position.clone(),
        line,
        file: sf.file.clone(),
        failure: true,
    };

    UserError { data: ued.into() }
}

