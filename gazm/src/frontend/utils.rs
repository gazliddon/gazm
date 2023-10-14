////////////////////////////////////////////////////////////////////////////////
use super::TSpan;

use thin_vec::{ ThinVec,thin_vec };

use unraveler::{ParseError,Parser, Collection, Splitter};
use grl_sources::Position;

pub fn to_pos(input: TSpan) -> Position {
    assert!(!input.is_empty());
    let p1 = input.first().unwrap().extra.as_range();
    let p2 = input.last().unwrap().extra.as_range();
    let r = p1.start .. p2.end;
    let p = Position::new(0, 0, r, grl_sources::AsmSource::FromStr);
    p
}

pub fn get_text(sp: TSpan) -> String {
    sp.first().unwrap().extra.get_text().to_owned()
}
pub fn get_str<'a>(sp: &'a TSpan<'a>) -> &'a str {
    sp.first().unwrap().extra.get_text()
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

