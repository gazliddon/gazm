////////////////////////////////////////////////////////////////////////////////
use super::TSpan;

use thin_vec::{thin_vec, ThinVec};

use grl_sources::Position;
use unraveler::{Collection, ParseError, Parser, Splitter};

pub fn to_pos(input: TSpan) -> Position {
    assert!(!input.is_empty());
    let p1 = input.first().unwrap().extra.as_range();
    let p2 = input.last().unwrap().extra.as_range();
    let r = p1.start..p2.end;
    Position::new(0, 0, r, grl_sources::AsmSource::FromStr)
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
    x.into_iter().chain(xxs.1).collect()
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

// used by test routines
use super::{to_tokens, Token};
use crate::item::{Item, Node};
use grl_sources::SourceFile;

pub fn get_items(node: &Node) -> (Item,ThinVec<Item>) {
    let items = node.children.iter().map(|c| c.item.clone()).collect();
    (node.item.clone(),items)
}

pub fn create_source_file(text: &str) -> SourceFile {
    grl_sources::SourceFile::new("No file", text, 0)
}

