#![deny(unused_imports)]

use crate::vars::VarsErrorKind;

use super::{to_pos, TSpan};

use emu6809::cpu::RegEnum;
use grl_sources::Position;
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

pub fn parse_failure(_txt: &str, _sp: TSpan) -> super::FrontEndError {
    panic!()
}
pub fn parse_error(_txt: &str, _sp: TSpan) -> super::FrontEndError {
    panic!()
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum FrontEndErrorKind {
    #[error("vars error {0}")]
    VarsError(#[from] Box<VarsErrorKind>),
    #[error("Parse error {0}")]
    ParseError(#[from] Box<ParseErrorKind>),
    #[error("You cannot define a macro inside a macro definition")]
    IllegalMacroDefinition,
    #[error("This opcode does not support this addressing mode")]
    OpcodeDoesNotSupportThisAddressingMode,
    #[error("Expected a register")]
    ExpectedARegister,
    #[error("Expected an index register")]
    ExpectedAnIndexRegister,
    #[error("Expected register {1} - got regsiter {0}")]
    ExpectedDifferentRegister(RegEnum,RegEnum),
    #[error("Unexpected duplicate register")]
    DuplicateRegisterInRegisterSet,
}

#[derive(Clone, Debug)]
pub struct FrontEndError {
    pub position: Position,
    pub kind: FrontEndErrorKind,
    pub severity: Severity,
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

    pub fn unsupported_addr_mode(sp: TSpan) -> Self {
        Self::new(
            sp,
            FrontEndErrorKind::OpcodeDoesNotSupportThisAddressingMode,
            Severity::Fatal,
        )
    }

    pub fn no_match_error(sp: TSpan) -> Self {
        Self::new(
            sp,
            FrontEndErrorKind::ParseError(ParseErrorKind::NoMatch.into()),
            Severity::Fatal,
        )
    }
}

impl<'a> ParseError<TSpan<'a>> for FrontEndError {
    fn from_error_kind(input: TSpan, kind: ParseErrorKind, severity: Severity) -> Self {
        Self {
            position: to_pos(input),
            kind: FrontEndErrorKind::ParseError(kind.into()),
            severity,
        }
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self {
        Self {
            kind: FrontEndErrorKind::ParseError(kind.into()),
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
