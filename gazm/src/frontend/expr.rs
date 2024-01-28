#![deny(unused_imports)]

use crate::assembler::AssemblerCpuTrait;

use unraveler::{alt, many0, match_span as ms, pair, sep_list, sep_list0};

use super::{
    concat, from_item_kids_tspan, from_item_tspan, parse_bracketed, GazmParser, Item, Node,
    PResult, TSpan,
    TokenKind::{self, *},
};

impl<C> GazmParser<C>
where
    C: AssemblerCpuTrait,
{
    pub fn op_to_node(
        input: TSpan,
        toke: TokenKind,
        item: Item<C::NodeKind>,
    ) -> PResult<Node<C::NodeKind>> {
        let (rest, (sp, _)) = ms(toke)(input)?;
        Ok((rest, from_item_tspan::<C>(item, sp)))
    }

    pub fn parse_expr_list0(input: TSpan) -> PResult<Vec<Node<C::NodeKind>>> {
        sep_list0(parse_expr::<C>, Comma)(input)
    }
    pub fn parse_expr_list(input: TSpan) -> PResult<Vec<Node<C::NodeKind>>> {
        sep_list(parse_expr::<C>, Comma)(input)
    }

    pub fn parse_term(input: TSpan) -> PResult<Node<C::NodeKind>> {
        alt((Self::parse_unary_term, Self::parse_non_unary_term))(input)
    }

    fn parse_unary_op(input: TSpan) -> PResult<Node<C::NodeKind>> {
        alt((
            |i| Self::op_to_node(i, Minus, Item::Sub),
            |i| Self::op_to_node(i, GreaterThan, Item::UnaryGreaterThan),
        ))(input)
    }

    fn parse_bracketed_expr(input: TSpan) -> PResult<Node<C::NodeKind>> {
        let (rest, (sp, mut matched)) = ms(parse_bracketed(parse_expr::<C>))(input)?;
        matched.item = Item::BracketedExpr;
        Ok((rest, Self::with_tspan(matched, sp)))
    }

    pub fn parse_non_unary_term(input: TSpan) -> PResult<Node<C::NodeKind>> {
        let parse_pc = |i| Self::op_to_node(i, Star, Item::Pc);
        alt((
            Self::parse_bracketed_expr,
            Self::parse_number,
            Self::parse_label,
            parse_pc,
        ))(input)
    }

    fn parse_unary_term(input: TSpan) -> PResult<Node<C::NodeKind>> {
        let (rest, (sp, (op, term))) =
            ms(pair(Self::parse_unary_op, Self::parse_non_unary_term))(input)?;
        let node = from_item_kids_tspan::<C>(Item::UnaryTerm, &[op, term], sp);
        Ok((rest, node))
    }

    fn parse_binary_op(input: TSpan) -> PResult<Node<C::NodeKind>> {
        alt((
            |i| Self::op_to_node(i, Plus, Item::Add),
            |i| Self::op_to_node(i, Minus, Item::Sub),
            |i| Self::op_to_node(i, Star, Item::Mul),
            |i| Self::op_to_node(i, Slash, Item::Div),
            |i| Self::op_to_node(i, Bar, Item::BitOr),
            |i| Self::op_to_node(i, Ampersand, Item::BitAnd),
            |i| Self::op_to_node(i, Caret, Item::BitXor),
            |i| Self::op_to_node(i, DoubleGreaterThan, Item::ShiftR),
            |i| Self::op_to_node(i, DoubleLessThan, Item::ShiftL),
        ))(input)
    }

    pub fn parse_op_term(input: TSpan) -> PResult<(Node<C::NodeKind>, Node<C::NodeKind>)> {
        let (rest, (op, term)) = pair(Self::parse_binary_op, Self::parse_term)(input)?;
        Ok((rest, (op, term)))
    }
}

pub fn parse_expr<C>(input: TSpan) -> PResult<Node<C::NodeKind>>
where
    C: AssemblerCpuTrait,
{
    let (rest, (sp, (term, vs))) = ms(pair(
        GazmParser::<C>::parse_term,
        many0(GazmParser::<C>::parse_op_term),
    ))(input)?;

    if vs.is_empty() && term.item.is_number() {
        Ok((rest, term))
    } else {
        let vs = vs.into_iter().flat_map(|(o, t)| [o, t]);
        let node = from_item_kids_tspan::<C>(Item::Expr, &concat((term, vs)), sp);
        Ok((rest, node))
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::frontend::*;
    use crate::opts::Opts;
    use item::{
        Item::{self, *},
        ParsedFrom::*,
    };
    use unraveler::Collection;

    #[test]
    fn test_expr() {
        use Item::*;
        let test = [
            ("3", Num(3, Decimal), vec![]),
            (
                "3 * 4 + 0x1 + (10  + 4)",
                Item::Expr,
                vec![
                    Num(3, Decimal),
                    Mul,
                    Num(4, Decimal),
                    Add,
                    Num(1, Hexadecimal),
                    Add,
                    BracketedExpr,
                ],
            ),
            ("-1 + -3", Expr, vec![UnaryTerm, Add, UnaryTerm]),
            ("1>>3", Expr, vec![Num(1, Decimal), ShiftR, Num(3, Decimal)]),
        ];
        let opts = Opts::default();

        for (text, i, wanted) in test.iter() {
            println!("Parsing {text}");
            let source_file = create_source_file(text);
            let tokens = to_tokens_no_comment(&source_file);
            let span = make_tspan(&tokens, &source_file, &opts);
            let (rest, matched) = parse_expr(span).unwrap();
            let (item, items) = get_items(&matched);
            println!("\tItem: {:?} : {:?}", item, items);
            assert_eq!(&item, i);
            assert!(rest.is_empty());
            assert_eq!(&items, wanted);
        }
    }
}
