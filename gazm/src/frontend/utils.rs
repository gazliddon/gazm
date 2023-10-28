#![deny(unused_imports)]
use crate::{
    item::{Item, Node, ParsedFrom},
    node::BaseNode,
};

use super::{FrontEndError, PResult, TSpan, TokenKind::*};
use grl_sources::{Position, SourceFile};
use thin_vec::{thin_vec, ThinVec};
use unraveler::{wrapped_cut, Collection, Parser, };

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
    let r = input.extra().get_pos(input);
    r
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

pub fn get_items(node: &Node) -> (Item, ThinVec<Item>) {
    let items = node.children.iter().map(|c| c.item.clone()).collect();
    (node.item.clone(), items)
}

pub fn create_source_file(text: &str) -> SourceFile {
    SourceFile::new("No file", text, 0)
}

pub fn parse_block<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError> + Copy,
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

pub fn take_line(full_span: TSpan) -> TSpan {
    match full_span.length() {
        0 | 1 => full_span,
        _ => {
            for i in 0..full_span.length() - 1 {
                let a = full_span.at(i).unwrap();
                let b = full_span.at(i + 1).unwrap();
                if a.extra.pos.line != b.extra.pos.line {
                    return full_span.take(i).expect("That's bad");
                }
            }
            full_span
        }
    }
}

pub fn parse_line<'a, P, O>(mut p: P) -> impl FnMut(TSpan<'a>) -> PResult<O> + Copy
where
    P: FnMut(TSpan<'a>) -> PResult<O> + Copy,
{
    move |i| {
        let line = take_line(i);
        let (rest, matched) = p.parse(line)?;
        Ok((rest, matched))
    }
}
