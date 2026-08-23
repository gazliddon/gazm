use crate::error::ParseError;
use crate::frontend::{
    err_error, err_fatal, AstNodeKind, CommandKind, FrontEndErrorKind, Node, PResult, TSpan,
};

use unraveler::{cut, match_span as ms, preceded, ParseErrorKind};

pub fn parse_commands(_input: TSpan) -> PResult<Node> {
    err_error(_input, ParseErrorKind::NoMatch)
}
