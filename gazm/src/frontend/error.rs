use super::TSpan;

pub type PResult<'a, T> = Result<(TSpan<'a>, T), MyError>;
use unraveler::{ Severity,ParseErrorKind,ParseError };

#[derive(Clone,Debug)]
pub enum MyError {
    ParseError(ParseErrorKind),
    Err,
}
impl<'a> ParseError<TSpan<'a>> for MyError {
    fn from_error_kind(_input: TSpan, _kind: ParseErrorKind, _sev: Severity) -> Self {
        Self::ParseError(_kind)
    }

    fn change_kind(self, _kind: ParseErrorKind) -> Self {
        Self::ParseError(_kind)
    }

    fn set_severity(self, _sev: Severity) -> Self {
        self
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn append(_input: TSpan, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}
