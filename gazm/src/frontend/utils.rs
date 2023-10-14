////////////////////////////////////////////////////////////////////////////////
use super::TSpan;

use thin_vec::{ ThinVec,thin_vec };

use unraveler::{ParseError,Parser, Collection, Splitter};

use grl_sources::Position;

pub fn to_pos(_input: TSpan) -> Position {
    assert!(!_input.is_empty());
    let p1 = _input.first().unwrap().extra.as_range();
    let _p2 = _input.last().unwrap().extra.as_range();

    let p3 = p1;
    let p = Position::new(0, 0, p3, grl_sources::AsmSource::FromStr);
    p
}

pub fn concat<I, II>(xxs: (I, II)) -> thin_vec::ThinVec<I>
where
    II: IntoIterator<Item = I>,
{
    let x = thin_vec::thin_vec![xxs.0];
    x.into_iter().chain(xxs.1.into_iter()).into_iter().collect()
}

pub fn match_span<P, I, O, E>(mut p: P) -> impl FnMut(I) -> Result<(I, (I, O)), E>
where
    P: Parser<I, O, E>,
    I: Splitter<E> + Collection,
    E: unraveler::ParseError<I>,
{
    move |i| {
        let (rest, matched) = p.parse(i.clone())?;
        let (matched_pos, _) = i.split_at(rest.length())?;
        Ok((rest, (matched_pos, matched)))
    }
}

