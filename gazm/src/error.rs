#![deny(unused_imports)]

use crate::{
    assembler::{self, AssemblerCpuTrait},
    frontend::{FrontEndError, FrontEndErrorKind},
    semantic::{AstNodeId, AstNodeRef},
    vars::VarsErrorKind,
};

use thin_vec::ThinVec;

use grl_sources::{
    grl_utils::FileError, EditErrorKind, Position, SourceFile, SourceFiles, SourceInfo,
    TextEditTrait,
};

use thiserror::Error;

pub type GResult<T> = Result<T, GazmErrorKind>;


#[derive(Error, Debug, Clone)]
pub enum GazmErrorKind {
    #[error("{0}")]
    UserErrors(NewErrorCollector<UserError>),
    #[error("{0}")]
    FrontEndErrors(NewErrorCollector<FrontEndError>),
    #[error(transparent)]
    FrontEndError(#[from] Box<FrontEndError>),
    #[error(transparent)]
    VarError(#[from] VarsErrorKind),
    #[error(transparent)]
    UserError(#[from] UserError),
    #[error("Misc: {0}")]
    Misc(String),
    #[error("Too Many Errors")]
    TooManyErrors(ErrorCollector),
    #[error(transparent)]
    BinaryError(#[from] assembler::BinaryError),
    #[error(transparent)]
    EditError(#[from] EditErrorKind),
    #[error(transparent)]
    FileError(#[from] FileError),
    #[error("Not implemented {0}")]
    NotImplemented(String)
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

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone)]
pub struct AstError {
    pub pos: Position,
    pub message: Option<String>,
    pub node_id: AstNodeId,
    pub failure: bool,
}

impl AstError {
    pub fn from_node<S,C: AssemblerCpuTrait>(msg: S, n: AstNodeRef<C>) -> Self
    where
        S: Into<String>,
        C: AssemblerCpuTrait
    {
        Self::from_node_id(msg, n.id(), n.value().pos)
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
impl crate::error::ErrorTrait for UserError {}

#[derive(PartialEq, Clone)]
pub struct UserError {
    pub data: Box<UserErrorData>,
}

#[derive(PartialEq, Clone, Error)]
pub struct UserWarning {
    pub data: Box<UserErrorData>,
}

impl std::fmt::Display for UserWarning {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!()
    }
}
impl std::fmt::Debug for UserWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(PartialEq, Clone)]
pub enum ErrorMessage {
    Plain(String),
    Markdown(String, String),
}

#[derive(PartialEq, Clone)]
pub struct UserErrorData {
    pub message: ErrorMessage,
    pub pos: Position,
    pub line: String,
    pub file: std::path::PathBuf,
    pub failure: bool,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.data.pretty().unwrap();
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
    pub fn new(message: &str, failure: bool, si: &SourceInfo) -> Self {
        Self {
            message: ErrorMessage::Plain(message.to_owned()),
            pos: si.pos,
            line: si.line_str.to_string(),
            file: si.file.clone(),
            failure,
        }
    }

    pub fn new_markdown(short: &str, full_text: &str, failure: bool, si: &SourceInfo) -> Self {
        Self {
            message: ErrorMessage::Markdown(short.to_owned(), full_text.to_owned()),
            pos: si.pos,
            line: si.line_str.to_string(),
            file: si.file.clone(),
            failure,
        }
    }

    pub fn from_text<S>(msg: S, info: &SourceInfo, failure: bool) -> Self
    where
        S: Into<String>,
    {
        Self::new(&msg.into(), failure, info)
    }

    pub fn from_front_end_error(err: &FrontEndError, sources: &SourceFiles) -> Self {
        let pos = &err.position;
        let si = sources.get_source_info(pos).unwrap();
        let failure = false;
        let message = err.to_string();
        Self::new(&message, failure, &si)
    }

    pub fn from_parse_error(err: &ParseError, sources: &SourceFiles) -> Self {
        let pos = &err.pos;
        let si = sources.get_source_info(pos).unwrap();
        let failure = err.failure;
        let message = &err.message();
        Self::new(message, failure, &si)
    }

    pub fn print_pretty(&self, _verbose_errors: bool) {
        use termimad::*;
        let skin = MadSkin::default();

        let pos = &self.pos;
        let (line, col) = pos.line_col_from_one();

        let line_num = format!("{line}");
        let spaces = " ".repeat(1 + line_num.len());
        let bar = format!("{spaces}|").info();
        let bar_line = format!("{line_num} |").info();

        let error = "\nError".bold().red();

        match &self.message {
            ErrorMessage::Plain(txt) => {
                println!("{error}: {}", txt.bold());
            }
            ErrorMessage::Markdown(short, _) => {
                println!("{error}: {}", short.bold())
            }
        }

        println!(
            "   {} {}:{}:{}",
            "-->".info(),
            self.file.to_string_lossy(),
            line,
            col
        );

        println!("{bar}");
        println!("{bar_line} {}", self.line);
        println!("{bar}{}^", " ".repeat(col));

        if _verbose_errors {
            if let ErrorMessage::Markdown(_, full_text) = &self.message {
                skin.print_text(full_text);
            }
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

        match &self.message {
            ErrorMessage::Plain(txt) => {
                writeln!(&mut s, "{error}: {}", txt.bold()).expect("lklkl");
            }
            ErrorMessage::Markdown(short, _) => writeln!(&mut s, "{short}").expect("kjkjk"),
        }

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

// impl AsRef<UserErrorData> for UserError {
//     fn as_ref(&self) -> &UserErrorData {
//         self.data.as_ref()
//     }
// }

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

    pub fn from_front_end_error(err: &FrontEndError, sources: &SourceFiles) -> Self {
        let data = UserErrorData::from_front_end_error(err, sources);
        data.into()
    }

    pub fn from_parse_error(err: &ParseError, sources: &SourceFiles) -> Self {
        let data = UserErrorData::from_parse_error(err, sources);
        data.into()
    }
}

////////////////////////////////////////////////////////////////////////////////
// UserErrors Collection

#[derive(Clone)]
pub struct ErrorCollector {
    _max_errors: usize,
    pub errors: ThinVec<GazmErrorKind>,
    _warnings: ThinVec<UserWarning>,
    errors_remaining: usize,
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self {
            _max_errors: 10,
            errors: Default::default(),
            _warnings: Default::default(),
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
            ..Default::default()
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
            panic!()
            // Err(GazmErrorKind::TooManyErrors(self.clone()))
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

    pub fn add_waring(&mut self, _warning: UserWarning) -> GResult<()> {
        unimplemented!()
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

use std::fmt::Display;
pub trait ErrorTrait: Display {}

impl ErrorTrait for FrontEndError {}

#[derive(Clone, Debug)]
pub struct NewErrorCollector<E>
where
    E: ErrorTrait,
{
    errors: Vec<E>,
    max_errors: usize,
}

impl<E> Default for NewErrorCollector<E>
where
    E: ErrorTrait,
{
    fn default() -> Self {
        Self {
            errors: Default::default(),
            max_errors: 20,
        }
    }
}

impl<E> Display for NewErrorCollector<E>
where
    E: ErrorTrait,
{
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.errors {
            write!(_f, "{e}")?;
        }
        Ok(())
    }
}

pub trait ErrorCollectorTrait: Sized {
    type Error;

    fn has_errors(&self) -> bool {
        self.num_of_errors() > 0
    }

    fn is_over_max_errors(&self) -> bool {
        self.num_of_errors() >= self.max_errors()
    }

    fn add<X: Into<Self::Error>>(&mut self, e: X);
    fn num_of_errors(&self) -> usize;
    fn max_errors(&self) -> usize;
    fn to_vec(self) -> Vec<Self::Error>;

    fn add_errors(&mut self, other: Self) {
        let errors = other.to_vec();
        for e in errors {
            self.add(e)
        }
    }

    fn to_result(self) -> Result<(), Self> {
        if self.has_errors() {
            Err(self)
        } else {
            Ok(())
        }
    }

    fn to_error(self) -> Result<(), Self> {
        if self.has_errors() {
            Err(self)
        } else {
            Ok(())
        }
    }
}

impl<E> ErrorCollectorTrait for NewErrorCollector<E>
where
    E: ErrorTrait,
{
    type Error = E;
    fn num_of_errors(&self) -> usize {
        self.errors.len()
    }

    fn max_errors(&self) -> usize {
        self.max_errors
    }

    fn to_vec(self) -> Vec<Self::Error> {
        self.errors
    }

    fn add<X: Into<Self::Error>>(&mut self, _err: X) {
        self.errors.push(_err.into());
    }
}

impl<E> NewErrorCollector<E>
where
    E: ErrorTrait,
{
    pub fn new(max_errors: usize) -> Self {
        Self {
            max_errors,
            errors: Default::default(),
        }
    }
}

fn get_line(sf: &SourceFile, line: isize) -> String {
    if line < 0 {
        String::new()
    } else {
        let txt = sf.get_text().get_line(line as usize).unwrap_or("");
        txt.to_owned()
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
            Markdown(short, full_text)
        }
        _ => Plain(format!("{e}")),
    };

    let line = e.position.line();
    let line = get_line(sf, line as isize);

    let ued = UserErrorData {
        message,
        pos: e.position,
        line,
        file: sf.file.clone(),
        failure: true,
    };

    UserError { data: ued.into() }
}

////////////////////////////////////////////////////////////////////////////////
