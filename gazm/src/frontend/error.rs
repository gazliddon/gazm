#![deny(unused_imports)]
use super::{ TSpan,to_pos };

pub type PResult<'a, T> = Result<(TSpan<'a>, T), FrontEndError>;

use unraveler::{ Severity,ParseErrorKind,ParseError };
use grl_sources::Position;
use thiserror::Error;

pub fn parse_failure(_txt: &str, _sp: TSpan) -> super::FrontEndError {
    panic!()
}
pub fn parse_error(_txt: &str, _sp: TSpan) -> super::FrontEndError {
    panic!()
}

#[derive(Clone,Debug,Error,PartialEq)]
pub enum FrontEndErrorKind {
    #[error("Parse error {0}")]
    ParseError(#[from] Box<ParseErrorKind>),
    #[error("You cannot define a macro inside a macro definition")]
    IllegalMacroDefinition,
}

#[derive(Clone,Debug)]
pub struct FrontEndError {
    position: Position,
    kind: FrontEndErrorKind,
    severity: Severity,
}

impl FrontEndError {
    pub fn new(sp: TSpan, kind: FrontEndErrorKind, severity: Severity) -> Self {
        let position= to_pos(sp);
        Self {
            kind, position, severity
        }
    }
}

impl<'a> ParseError<TSpan<'a>> for FrontEndError {
    fn from_error_kind(input: TSpan, kind: ParseErrorKind, severity: Severity) -> Self {
        Self {
            position: to_pos(input),
            kind : FrontEndErrorKind::ParseError(kind.into()),
            severity,
        }
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self {
        Self {
            kind : FrontEndErrorKind::ParseError(kind.into()),
            ..self
        }
    }

    fn set_severity(self, severity: Severity) -> Self {
        Self {
            severity,
            ..self
        }
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn append(_input: TSpan, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}

