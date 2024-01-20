#![deny(unused_imports)]
use super::{
    BaseNode, FrontEndError, FrontEndErrorKind, IdentifierKind, Item, Node, PResult, ParsedFrom,
    TSpan, TokenKind::*,
};

use grl_sources::{Position, SourceFile};
use thin_vec::{thin_vec, ThinVec};
use unraveler::{
    map, match_span as ms, tag, wrapped_cut, Collection, ParseErrorKind, Parser, Severity,
};

pub fn mk_pc_equate(node: &Node) -> Node {
    use Item::{AssignmentFromPc, Label, LocalAssignmentFromPc, LocalLabel};
    let pos = node.ctx;

    match &node.item {
        Label(label_def) => Node::new(AssignmentFromPc(label_def.clone()), pos),
        LocalLabel(label_def) => Node::new(LocalAssignmentFromPc(label_def.clone()), pos),
        _ => panic!("shouldn't happen"),
    }
}

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
        Node::from_item_tspan(Item::from_number(num, ParsedFrom::Expression), sp)
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
    SourceFile::new("No file", text, grl_sources::AsmSource::FileId(0))
}

fn wrong_termination(e: FrontEndError, kind: FrontEndErrorKind) -> FrontEndError {
    use FrontEndErrorKind::*;
    match e.kind {
        ParseError(ParseErrorKind::MissingWrapTerminator) => FrontEndError { kind, ..e },
        _ => e,
    }
}

pub fn parse_block<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError> + Copy,
{
    use FrontEndErrorKind::*;
    move |i| {
        wrapped_cut(OpenBrace, p, CloseBrace)(i).map_err(|e| wrong_termination(e, NoCloseBrace))
    }
}

pub fn parse_bracketed<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    use FrontEndErrorKind::*;
    move |i| {
        wrapped_cut(OpenBracket, p, CloseBracket)(i)
            .map_err(|e| wrong_termination(e, NoCloseBracket))
    }
}

pub fn parse_sq_bracketed<'a, O, P>(p: P) -> impl Fn(TSpan<'a>) -> PResult<O> + Copy
where
    P: Parser<TSpan<'a>, O, FrontEndError>,
{
    use FrontEndErrorKind::*;
    move |i| {
        wrapped_cut(OpenSquareBracket, p, CloseSquareBracket)(i)
            .map_err(|e| wrong_termination(e, NoCloseSqBracket))
    }
}

/// Split this span at the next line
/// used for error recovery
pub fn split_at_next_line(full_span: TSpan) -> PResult<TSpan> {
    use FrontEndErrorKind::*;
    let this_line = take_line(full_span);
    let (rest, matched) = full_span
        .split(this_line.length())
        .map_err(|_| FrontEndError::new(this_line, UnableToFindNextLine, Severity::Error))?;
    Ok((rest, matched))
}

pub fn take_line(full_span: TSpan) -> TSpan {
    match full_span.length() {
        0 | 1 => full_span,
        _ => {
            for i in 0..full_span.length() - 1 {
                let a = full_span.at(i).unwrap();
                let b = full_span.at(i + 1).unwrap();
                if a.extra.pos.line() != b.extra.pos.line() {
                    return full_span.take(i + 1).expect("That's bad");
                }
            }
            full_span
        }
    }
}

pub fn parse_line<'a, P, O>(p: P) -> impl FnMut(TSpan<'a>) -> PResult<O> + Copy
where
    P: FnMut(TSpan<'a>) -> PResult<O> + Copy,
{
    move |i| {
        // grab 1 lines worth
        let line = take_line(i);
        // parse that line and capture what was parsed
        let (_, (sp, matched)) = ms(p)(line)?;
        // drop however many we parsed
        let new_span = i.drop(sp.length()).unwrap();
        Ok((new_span, matched))
    }
}

/// Get's a string if this is label
pub fn get_label_string(input: TSpan) -> PResult<String> {
    use IdentifierKind::*;
    map(tag(Identifier(Label)), |sp| get_text(sp))(input)
}

#[allow(unused_imports)]
mod test {

    use super::*;
    use crate::frontend::*;
    use crate::opts::Opts;

    use unraveler::*;

    #[test]
    fn test_parse_line() {
        let opts = Opts::default();
        let text = r#"Line1 Line2
        Line2"#;

        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);

        let ts: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", ts);

        let span = make_tspan(&tokens, &sf, &opts);

        let x = take_line(span);
        let ts: Vec<_> = x.as_slice().iter().map(|t| t.kind).collect();
        println!("took line {:?}", ts);

        // assert!(false)
    }
}
