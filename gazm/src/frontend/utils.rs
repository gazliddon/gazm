////////////////////////////////////////////////////////////////////////////////
use super::{ TSpan,FrontEndError };

use thin_vec::{thin_vec, ThinVec};

use grl_sources::Position;
use unraveler::{Collection, ParseError, Parser, Splitter};

pub fn to_pos(input: TSpan) -> Position {
    assert!(!input.is_empty());

    let extra_start = &input.first().unwrap().extra;
    let extra_end = &input.last().unwrap().extra;

    let r = extra_start.as_range().start..extra_end.as_range().end;
    let tp = extra_start.as_text_pos();
    let file = extra_start.source_file.file_id;

    Position::new(
        tp.line(),
        tp.char(),
        r,
        grl_sources::AsmSource::FileId(file),
    )
}

pub fn get_text(sp: TSpan) -> String {
    get_str(&sp).to_owned()
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

pub fn match_span<P, I, O, E>(mut p: P) -> impl FnMut(I) -> Result<(I, (I, O)), E> + Copy + Clone
where
    I : Clone + Copy,
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

pub fn get_items(node: &Node) -> (Item, ThinVec<Item>) {
    let items = node.children.iter().map(|c| c.item.clone()).collect();
    (node.item.clone(), items)
}

pub fn create_source_file(text: &str) -> SourceFile {
    grl_sources::SourceFile::new("No file", text, 0)
}

use super::{ TokenKind, PResult };
use unraveler::wrapped_cut;

pub fn parse_block<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    use TokenKind::{CloseBrace, OpenBrace};
    move |i| wrapped_cut(OpenBrace, p, CloseBrace)(i)
}
pub fn parse_bracketed<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    use TokenKind::{CloseBracket, OpenBracket};
    move |i| wrapped_cut(OpenBracket, p, CloseBracket)(i)
}

