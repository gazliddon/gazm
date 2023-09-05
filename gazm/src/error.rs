use std::path::PathBuf;

use crate::{
    ast::{AstNodeId, AstNodeRef},
    binary,
    parse::locate::{span_to_pos, Span},
};

use utils::{
    sources::{EditErrorKind, Position, SourceErrorType, SourceInfo},
    SearchError,
};

use thiserror::Error;

pub struct InternalErrorKind {
    msg: String,
    file: PathBuf,
    line: usize,
    col: usize,
}

pub type GResult<T> = Result<T, GazmErrorKind>;

#[derive(Error, Debug, Clone)]
pub enum GazmErrorKind {
    #[error(transparent)]
    UserError(#[from] UserError),
    #[error("Misc: {0}")]
    Misc(String),
    #[error("Too Many Errors")]
    TooManyErrors(ErrorCollector),
    #[error(transparent)]
    BinaryError(#[from] binary::BinaryError),
    #[error("Parse error {0}")]
    ParseError(#[from] Box<ParseError>),
    #[error(transparent)]
    EditError(#[from] EditErrorKind),
    #[error(transparent)]
    FileError(#[from] SearchError),
    #[error(transparent)]
    SourceError(#[from] SourceErrorType),
}

impl From<String> for GazmErrorKind {
    fn from(x: String) -> Self {
        GazmErrorKind::Misc(x)
    }
}
impl From<anyhow::Error> for GazmErrorKind {
    fn from(x: anyhow::Error) -> Self {
        GazmErrorKind::Misc(x.to_string())
    }
}

// Anyhow, don't care what the error type is.
// application should use this
// thiserror = typed errors, gasmlib
#[derive(Error, Debug, PartialEq, Clone)]
pub struct ParseError {
    pub message: Option<String>,
    pub pos: Position,
    pub failure: bool,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parsing! {:?}", self.message())
    }
}

impl ParseError {
    pub fn message(&self) -> String {
        self.message.clone().unwrap_or_default()
    }
}

pub fn parse_error(err: &str, ctx: Span) -> nom::Err<ParseError> {
    nom::Err::Error(ParseError::new(err.to_string(), &ctx, false))
}

pub fn parse_failure(err: &str, ctx: Span) -> nom::Err<ParseError> {
    nom::Err::Failure(ParseError::new(err.to_string(), &ctx, true))
}

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError>;

impl ParseError {
    pub fn new(message: String, span: &Span, failure: bool) -> ParseError {
        Self::new_from_pos(message.as_str(), &span_to_pos(*span), failure)
    }

    pub fn new_from_pos(message: &str, pos: &Position, failure: bool) -> Self {
        Self {
            message: Some(message.to_owned()),
            pos: pos.clone(),
            failure,
        }
    }

    pub fn set_failure(self, failure: bool) -> Self {
        let mut ret = self;
        ret.failure = failure;
        ret
    }

    pub fn is_failure(&self) -> bool {
        self.failure
    }
}

impl From<nom::Err<ParseError>> for ParseError {
    fn from(i: nom::Err<ParseError>) -> Self {
        match i {
            nom::Err::Incomplete(_) => panic!(),
            nom::Err::Error(e) => e.set_failure(false),
            nom::Err::Failure(e) => e.set_failure(true),
        }
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for ParseError {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {kind:?}"), &input, false)
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{c}'"), &input, false)
    }
}

// That's what makes it nom-compatible.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone)]
pub struct AstError {
    pub pos: Position,
    pub message: Option<String>,
    pub node_id: AstNodeId,
    pub failure: bool,
}

impl AstError {
    pub fn from_node<S>(msg: S, n: AstNodeRef) -> Self
    where
        S: Into<String>,
    {
        Self::from_node_id(msg, n.id(), n.value().pos.clone())
    }

    pub fn from_node_id<S>(msg: S, id: AstNodeId, pos: Position) -> Self
    where
        S: Into<String>,
    {
        Self {
            pos,
            message: Some(msg.into()),
            node_id: id,
            failure: false,
        }
    }
}

impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, col) = self.pos.line_col_from_one();

        let err_string = self
            .message
            .clone()
            .unwrap_or_else(|| "NO ERROR".to_string());
        write!(f, "{err_string} : ({line}:{col})")
    }
}

////////////////////////////////////////////////////////////////////////////////
// User Error

impl std::error::Error for UserError {}

#[derive(PartialEq, Clone)]
pub struct UserError {
    pub data: Box<UserErrorData>,
}

#[derive(PartialEq, Clone)]
pub struct UserErrorData {
    pub message: String,
    pub pos: Position,
    pub fragment: String,
    pub line: String,
    pub file: std::path::PathBuf,
    pub failure: bool,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.pretty().unwrap();
        write!(f, "{s}")
    }
}
impl std::fmt::Debug for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
use crate::messages::Messageize;
use colored::*;

impl UserErrorData {
    pub fn from_text<S>(msg: S, info: &SourceInfo, is_failure: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            message: msg.into(),
            pos: info.pos.clone(),
            fragment: info.fragment.to_string(),
            line: info.line_str.to_string(),
            file: info.source_file.file.clone(),
            failure: is_failure,
        }
    }

    pub fn from_parse_error(err: &ParseError, sources: &utils::sources::SourceFiles) -> Self {
        let si = sources.get_source_info(&err.pos).unwrap();

        Self {
            message: err.message(),
            pos: err.pos.clone(),
            fragment: si.fragment.to_string(),
            line: si.line_str.to_string(),
            file: si.file,
            failure: err.failure,
        }
    }

    pub fn pretty(&self) -> GResult<String> {
        use std::fmt::Write as FmtWrite;

        let mut s = String::new();

        let pos = &self.pos;
        let (line, col) = pos.line_col_from_one();

        let line_num = format!("{line}");
        let spaces = " ".repeat(1 + line_num.len());
        let bar = format!("{spaces}|").info();
        let bar_line = format!("{line_num} |").info();

        let error = "error".bold().red();

        writeln!(&mut s, "{error}: {}", self.message.bold()).expect("kj");

        writeln!(
            &mut s,
            "   {} {}:{}:{}",
            "-->".info(),
            self.file.to_string_lossy(),
            line,
            col
        )
        .expect("kj");

        writeln!(s, "{bar}").expect("kj");
        writeln!(s, "{bar_line} {}", self.line).expect("kj");
        writeln!(s, "{bar}{}^", " ".repeat(col)).expect("kj");
        Ok(s)
    }
}

impl AsRef<UserErrorData> for UserError {
    fn as_ref(&self) -> &UserErrorData {
        self.data.as_ref()
    }
}

impl From<UserErrorData> for UserError {
    fn from(value: UserErrorData) -> Self {
        Self { data: value.into() }
    }
}

impl UserError {
    pub fn from_ast_error(error: AstError, info: &SourceInfo) -> Self {
        let message = error.message.unwrap_or_else(|| "Error".to_string());
        Self::from_text(message, info, error.failure)
    }

    pub fn from_text<S>(msg: S, info: &SourceInfo, is_failure: bool) -> Self
    where
        S: Into<String>,
    {
        let data = UserErrorData::from_text(msg, info, is_failure);
        data.into()
    }

    pub fn pretty(&self) -> GResult<String> {
        self.data.pretty()
    }

    pub fn from_parse_error(err: &ParseError, sources: &utils::sources::SourceFiles) -> Self {
        let data = UserErrorData::from_parse_error(err, sources);
        data.into()
    }
}

////////////////////////////////////////////////////////////////////////////////
// UserErrors Collection

#[derive(Clone)]
pub struct ErrorCollector {
    _max_errors: usize,
    pub errors: thin_vec::ThinVec<GazmErrorKind>,
    errors_remaining: usize,
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self {
            _max_errors: 10,
            errors: Default::default(),
            errors_remaining: Default::default(),
        }
    }
}

impl std::fmt::Display for ErrorCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in &self.errors {
            writeln!(f, "{x}")?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for ErrorCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for ErrorCollector {}

impl ErrorCollector {
    pub fn new(_max_errors: usize) -> Self {
        Self {
            _max_errors,
            errors_remaining: _max_errors,
            errors: Default::default(),
        }
    }

    pub fn check_errs<T, F>(&mut self, mut f: F) -> GResult<()>
    where
        F: FnMut() -> GResult<T>,
    {
        if let Err(e) = f() {
            self.add_error(e, false)?;
        }
        Ok(())
    }

    pub fn add_errors(&mut self, other: Self) {
        for x in other.errors.into_iter() {
            self.errors.push(x)
        }
    }

    pub fn num_of_errors(&self) -> usize {
        self.errors.len()
    }

    pub fn has_errors(&self) -> bool {
        self.num_of_errors() != 0
    }

    pub fn raise_errors(&self) -> GResult<()> {
        if self.has_errors() {
            Err(GazmErrorKind::TooManyErrors(self.clone()))
        } else {
            Ok(())
        }
    }
    pub fn add_result<T>(&mut self, e: GResult<T>) -> GResult<()> {
        if let Err(err) = e {
            self.add_error(err, false)
        } else {
            Ok(())
        }
    }

    pub fn add_user_error(&mut self, err: UserError) -> GResult<()> {
        let failure = err.as_ref().failure;
        let err = GazmErrorKind::UserError(err);
        self.add_error(err, failure)
    }

    pub fn add_error(&mut self, err: GazmErrorKind, failure: bool) -> GResult<()> {
        self.errors.push(err.clone());

        if self.errors_remaining == 0 || failure {
            Err(err)
        } else {
            self.errors_remaining -= 1;
            Ok(())
        }
    }

    pub fn add_ast_error(&mut self, err: AstError, info: &SourceInfo) -> GResult<()> {
        self.add_user_error(UserError::from_ast_error(err, info))
    }

    pub fn add_text_error<S>(&mut self, msg: S, info: &SourceInfo, is_failure: bool) -> GResult<()>
    where
        S: Into<String>,
    {
        self.add_user_error(UserError::from_text(msg, info, is_failure))
    }
}

////////////////////////////////////////////////////////////////////////////////
