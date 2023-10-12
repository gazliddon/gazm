use super::TSpan;

pub type PResult<'a, T> = Result<(TSpan<'a>, T), MyError>;
use unraveler::{ Severity,ParseErrorKind,ParseError };

#[derive(Clone)]
pub enum MyError {
    Err,
}
impl<'a> ParseError<TSpan<'a>> for MyError {
    fn from_error_kind(_input: TSpan, _kind: ParseErrorKind, _sev: Severity) -> Self {
        todo!()
    }

    fn change_kind(self, _kind: ParseErrorKind) -> Self {
        todo!()
    }

    fn set_severity(self, _sev: Severity) -> Self {
        todo!()
    }

    fn severity(&self) -> Severity {
        todo!()
    }

    fn append(_input: TSpan, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}
