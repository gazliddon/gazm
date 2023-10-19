#![deny(unused_imports)]
////////////////////////////////////////////////////////////////////////////////
use thin_vec::{thin_vec, ThinVec};
use unraveler::{Collection, Parser, Splitter};
use grl_sources::{ Position, SourceFile,AsmSource::FileId };

use crate::item::{Item, Node};

use super::{PResult, TokenKind::*,FrontEndError, TSpan};

impl crate::node::BaseNode<Item, Position> {
    pub fn from_item_tspan(item: Item, sp: TSpan) -> Self {
        Self::from_item_pos(item, to_pos(sp))
    }

    pub fn from_item_kids_tspan(item: Item, kids: &[Node], sp: TSpan) -> Self {
        Self::new_with_children(item, kids, to_pos(sp))
    }
    pub fn from_item_kid_tspan(item: Item, kid: Node, sp: TSpan) -> Self {
        Self::new_with_children(item, &[kid], to_pos(sp))
    }

    pub fn from_num_tspan(num: i64, sp: TSpan) -> Self {
        Node::from_item_tspan(
            Item::from_number(num, crate::item::ParsedFrom::FromExpr),
            sp,
        )
    }
    pub fn with_tspan(self, sp: TSpan) -> Self {
        let mut ret = self;
        ret.ctx = to_pos(sp);
        ret
    }
}

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
        FileId(file),
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
    I: Clone + Copy,
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

pub fn get_items(node: &Node) -> (Item, ThinVec<Item>) {
    let items = node.children.iter().map(|c| c.item.clone()).collect();
    (node.item.clone(), items)
}

pub fn create_source_file(text: &str) -> SourceFile {
    SourceFile::new("No file", text, 0)
}

use unraveler::wrapped_cut;

pub fn parse_block<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    move |i| wrapped_cut(OpenBrace, p, CloseBrace)(i)
}
pub fn parse_bracketed<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    move |i| wrapped_cut(OpenBracket, p, CloseBracket)(i)
}

pub fn parse_sq_bracketed<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    move |i| wrapped_cut(OpenSquareBracket, p, CloseSquareBracket)(i)
}
