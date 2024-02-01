use crate::error::ParseError;
use crate::frontend::{CommandKind, AstNodeKind, PResult, TSpan, FrontEndErrorKind, err_fatal, err_error};

use crate::cpu6800::Node;

use unraveler::{cut, match_span as ms, preceded, ParseErrorKind, };

pub fn parse_commands(_input: TSpan) -> PResult<Node> {
    err_error(_input, ParseErrorKind::NoMatch)
}
