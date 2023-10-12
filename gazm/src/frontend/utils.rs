////////////////////////////////////////////////////////////////////////////////
use super::TSpan;

use grl_sources::Position;

pub fn to_pos(_input: TSpan) -> Position {
    assert!(!_input.is_empty());
    let p1 = _input.first().unwrap().extra.as_range();
    let _p2 = _input.last().unwrap().extra.as_range();

    let p3 = p1;
    let p = Position::new(0, 0, p3, grl_sources::AsmSource::FromStr);
    p
}
