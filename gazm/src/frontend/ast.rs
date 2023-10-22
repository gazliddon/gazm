use std::{
    path::{Path, PathBuf},
    process::CommandArgs,
};

use crate::{
    async_tokenize::IncludeErrorKind,
    cli::parse_command_line,
    error::IResult,
    item::{Item, LabelDefinition, Node, ParsedFrom},
    item6809::{IndexParseType, MC6809::SetDp},
    parse::locate::span_to_pos,
};

use thin_vec::{thin_vec, ThinVec};

use super::{TSpan,CommandKind, IdentifierKind, TokenKind, PResult};
use IdentifierKind::Command;

fn to_ast(_tokes: &[TokenKind], _txt: &str) {}
