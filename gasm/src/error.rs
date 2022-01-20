use std::fmt::Display;
use std::path::PathBuf;

use nom::{self, Offset};

use crate::ast::{AstNodeId, AstNodeRef};
use crate::locate::Span;
use crate::position::Position;
use crate::sourcefile::SourceInfo;
use nom::AsBytes;

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    pub span: Span<'a>,
    pub message: Option<String>,
}

impl<'a> ParseError<'a> {
    pub fn message(&self) -> String {
        self.message.clone().unwrap_or_else(|| "".to_string())
    }
}

pub fn error<'a>(err: &str, ctx: Span<'a>) -> nom::Err<ParseError<'a>> {
    nom::Err::Error(ParseError::new(err.to_string(), &ctx))
}
pub fn user_error(_err: &str, _ctx: Span) -> UserError {
    panic!()
}

pub fn failure<'a>(err: &str, ctx: Span<'a>) -> nom::Err<ParseError<'a>> {
    nom::Err::Failure(ParseError::new(err.to_string(), &ctx))
}

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;

impl<'a> ParseError<'a> {
    pub fn new(message: String, span: &Span<'a>) -> ParseError<'a> {
        Self {
            span: *span,
            message: Some(message),
        }
    }

    // pub fn span(&self) -> &Span { &self.span }

    pub fn line(&self) -> usize {
        panic!()
    }

    pub fn offset(&self) -> usize {
        panic!()
    }

    pub fn fragment(&self) -> &'a str {
        &self.span
    }

    // pub fn from_text(message : &str) -> Self {
    //     Self {message: Some(message.to_string()),
    //     pos : Default::default() }
    // }
}

impl<'a> From<nom::Err<ParseError<'a>>> for ParseError<'a> {
    fn from(i: nom::Err<ParseError<'a>>) -> Self {
        match i {
            nom::Err::Incomplete(_) => panic!(),
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        }
    }
}

// That's what makes it nom-compatible.
impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), &input)
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), &input)
    }
}
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone)]
pub struct AstError {
    pub pos: Position,
    pub message: Option<String>,
    pub node_id: AstNodeId,
}

impl AstError {
    pub fn from_node<S>(msg: S, n: AstNodeRef) -> Self
    where
        S: Into<String>,
    {
        Self {
            pos: n.value().pos.clone(),
            message: Some(msg.into()),
            node_id: n.id(),
        }
    }
}

impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_string = self.message.clone().unwrap_or("NO ERROR".to_string());
        write!(f, "{} : ({}:{})", err_string, self.pos.line, self.pos.col)
    }
}

impl std::error::Error for AstError {}

////////////////////////////////////////////////////////////////////////////////
impl std::error::Error for UserError { }

#[derive(PartialEq, Clone)]
pub struct UserError {
    pub message: String,
    pub pos: Position,
    pub fragment: String,
    pub line: String,
    pub file: std::path::PathBuf,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.pretty().unwrap();
        write!( f, "{}",s)
    }
}

impl std::fmt::Debug for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self,f)
    }
}

use crate::messages::Messageize;
use colored::*;

impl UserError {
    pub fn from_ast_error(_err: AstError, info: &SourceInfo) -> Self {
        let message = _err.message.unwrap_or_else(|| "Error".to_string());
        Self::from_text(message, info, &_err.pos)
    }

    pub fn from_text<S>(msg: S, info: &SourceInfo, pos: &Position) -> Self
    where
        S: Into<String>,
    {
        Self {
            message: msg.into(),
            pos: pos.clone(),
            fragment: info.fragment.to_string(),
            line: info.line_str.to_string(),
            file: info.source_file.file.clone(),
        }
    }

    pub fn pretty(&self) -> Result<String, Box<dyn std::error::Error>> {
        use std::fmt::Write as FmtWrite;
        use std::io::Write as IoWrite;

        let mut s = String::new();

        let pos = &self.pos;
        let line = pos.line;
        let col = pos.col;

        let line_num = format!("{}", line);
        let spaces = " ".repeat(1 + line_num.len());
        let bar = format!("{}|", spaces).info();
        let bar_line = format!("{} |", line_num).info();

        writeln!(&mut s, "{}: {}", "error".error(), self.message.bold())?;
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

    pub fn from_parse_error(err: ParseError, file: &std::path::Path) -> Self {
        let line = err.span.get_line_beginning();
        let line = String::from_utf8_lossy(line).to_string();
        Self {
            message: err.message(),
            pos: err.span.into(),
            fragment: err.span.to_string(),
            line,
            file: file.to_path_buf(),
        }
    }
}
