#![deny(unused_imports)]
use crate::{
    item::{Item, Node, ParsedFrom},
    node::BaseNode,
};

use super::{FrontEndError, PResult, TSpan, TokenKind::*};
use grl_sources::{Position, SourceFile};
use thin_vec::{thin_vec, ThinVec};
use unraveler::{wrapped_cut, Collection, Parser, Splitter, ParseError};

impl BaseNode<Item, Position> {
    pub fn block(items: ThinVec<Self>, sp: TSpan) -> Self {
        Self::from_item_tspan(Item::Block, sp).with_children_vec(items)
    }

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
        Node::from_item_tspan(Item::from_number(num, ParsedFrom::FromExpr), sp)
    }

    pub fn with_tspan(self, sp: TSpan) -> Self {
        let mut ret = self;
        ret.ctx = to_pos(sp);
        ret
    }
}

pub fn to_pos(input: TSpan) -> Position {
    input.extra().get_pos(input)
}

pub fn get_text(sp: TSpan) -> String {
    get_str(&sp).to_owned()
}

pub fn get_str<'a>(sp: &'a TSpan<'a>) -> &'a str {
    sp.extra().get_str(*sp)
}

pub fn concat<I, II>(xxs: (I, II)) -> ThinVec<I>
where
    II: IntoIterator<Item = I>,
{
    let x = thin_vec![xxs.0];
    x.into_iter().chain(xxs.1).collect()
}

pub fn match_span<P, I, O, E>(mut p: P) -> impl FnMut(I) -> Result<(I, (I, O)), E> + Copy + Clone
where
    I: Clone + Copy,
    P: Parser<I, O, E>,
    I: Splitter<E> + Collection,
    E: ParseError<I>,
{
    move |i| {
        let (rest, matched) = p.parse(i)?;
        let matched_len = i.length() - rest.length();
        let matched_span = i.take(matched_len)?;
        Ok((rest, (matched_span, matched)))
    }
}

pub fn get_items(node: &Node) -> (Item, ThinVec<Item>) {
    let items = node.children.iter().map(|c| c.item.clone()).collect();
    (node.item.clone(), items)
}

pub fn create_source_file(text: &str) -> SourceFile {
    SourceFile::new("No file", text, 0)
}

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


pub fn take_line(full_span: TSpan) -> (TSpan, TSpan) {
    let f = || {
        for i in 1..full_span.length() {
            let (rest, matched) = full_span.split(i).expect("That's bad");
            let mpos = &matched.first().unwrap().extra.pos;
            let rpos = &rest.first().unwrap().extra.pos;
            if mpos.line != rpos.line {
                return (rest, matched);
            }
        }

        full_span.split(full_span.length()).unwrap()
    };

    let (rest, matched) = match full_span.length() {
        0 => (full_span, full_span),
        1 => full_span.split(1).unwrap(),
        _ => f(),
    };

    (rest, matched)
}

pub fn parse_line_parser<'a, P>(input: TSpan<'a>, mut p: P) -> PResult<Node>
where
    P: FnMut(TSpan<'a>) -> PResult<Node> + Copy,
{
    let (rest, line) = take_line(input);
    let (_, matched) = p(line)?;
    Ok((rest, matched))
}

pub fn parse_line<'a, P>(p: P) -> impl FnMut(TSpan<'a>) -> PResult<Node> + Copy
where
    P: FnMut(TSpan<'a>) -> PResult<Node> + Copy,
{
    move |i| parse_line_parser(i, p)
}

