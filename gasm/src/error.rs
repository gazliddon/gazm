
use crate::ast::{AstNodeId, AstNodeRef};
use crate::locate::span_to_pos;
use crate::locate::Span;
use romloader::sources::{Position, SourceInfo};

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub message: Option<String>,
    pub pos: Position,
    pub failure: bool,
}

impl ParseError {
    pub fn message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "".to_string())
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
    pub fn new(message: String, span: &Span, failure : bool) -> ParseError {
        Self {
            message: Some(message),
            pos: span_to_pos(*span),
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

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), &input, false)
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), &input, false)
    }
}

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
        Self::from_node_id(
            msg,
            n.id(),
            n.value().pos.clone()
        )
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
        let err_string = self
            .message
            .clone()
            .unwrap_or_else(|| "NO ERROR".to_string());
        write!(f, "{} : ({}:{})", err_string, self.pos.line, self.pos.col)
    }
}

impl std::error::Error for AstError {}

////////////////////////////////////////////////////////////////////////////////
// User Error

impl std::error::Error for UserError {}

#[derive(PartialEq, Clone)]
pub struct UserError {
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
        write!(f, "{}", s)
    }
}
impl std::fmt::Debug for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
use crate::messages::Messageize;
use colored::*;

impl UserError {
    pub fn from_ast_error(_err: AstError, info: &SourceInfo) -> Self {
        let message = _err.message.unwrap_or_else(|| "Error".to_string());
        Self::from_text(message, info, _err.failure)
    }

    pub fn from_text<S>(msg: S, info: &SourceInfo, is_failure : bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            message: msg.into(),
            pos: info.pos.clone(),
            fragment: info.fragment.to_string(),
            line: info.line_str.to_string(),
            file: info.source_file.file.clone(),
            failure : is_failure,
        }
    }

    pub fn pretty(&self) -> Result<String, Box<dyn std::error::Error>> {
        use std::fmt::Write as FmtWrite;

        let mut s = String::new();

        let pos = &self.pos;
        let line = pos.line;
        let col = pos.col;

        let line_num = format!("{}", line);
        let spaces = " ".repeat(1 + line_num.len());
        let bar = format!("{}|", spaces).info();
        let bar_line = format!("{} |", line_num).info();

        writeln!(&mut s, "{}", self.message.bold())?;
        writeln!(
            &mut s,
            "   {} {}:{}:{}",
            "-->".info(),
            self.file.to_string_lossy(),
            line,
            col
        )?;
        writeln!(s, "{}", bar)?;
        writeln!(s, "{} {}", bar_line, self.line)?;
        writeln!(s, "{}{}^", bar, " ".repeat(self.pos.col))?;
        Ok(s)
    }

    pub fn from_parse_error(
        err: ParseError,
        sources: &romloader::sources::Sources,
    ) -> Self {
        let si = sources.get_source_info(&err.pos).unwrap();

        Self {
            message: err.message(),
            pos: err.pos,
            fragment: si.fragment.to_string(),
            line: si.line_str.to_string(),
            file: si.file,
            failure: err.failure
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// UserErrors Collection

#[derive(PartialEq, Clone)]
pub struct UserErrors {
    max_errors: usize,
    errors: Vec<UserError>,
    errors_remaining : usize,
}

impl std::fmt::Display for UserErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in &self.errors {
            write!(f, "{}", x)?;
        }
        Ok(())
    }
}
impl std::fmt::Debug for UserErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for UserErrors {}

impl UserErrors {
    pub fn new(max_errors : usize) -> Self {
        Self {
            max_errors,
            errors_remaining : max_errors,
            errors : vec![]
        }
    }

    pub fn add_errors(&mut self, other : Self) {
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

    pub fn raise_errors(&self) -> Result<(), UserError> {
        if self.has_errors() {
            Err(self.errors.last().unwrap().clone())
        } else {
            Ok(())
        }
    }

    pub fn add_error(&mut self, err : UserError) -> Result<(), UserError>{
        let failure = err.failure;
        self.errors.push(err);

        if self.errors_remaining == 0 || failure {
            Err(self.errors.last().unwrap().clone())
        } else {
            self.errors_remaining -= 1;
            Ok(())
        }
    }

    pub fn add_ast_error(&mut self, err: AstError, info: &SourceInfo) -> Result<(),UserError>{
        self.add_error(UserError::from_ast_error(err, info))
    }

    pub fn add_parse_error(
        &mut self,
        err: ParseError,
        sources: &romloader::sources::Sources,
    ) -> Result<(), UserError>{
        self.add_error(UserError::from_parse_error(err, sources))
    }

    pub fn add_text_error<S>(&mut self, msg: S, info: &SourceInfo, is_failure : bool) -> Result<(),UserError>
    where
        S: Into<String>,
    {
        self.add_error(UserError::from_text(msg, info, is_failure))
    }
}

////////////////////////////////////////////////////////////////////////////////


