#![deny(unused_imports)]

use unraveler::{many0, match_span as ms, pair, preceded, sep_list0, tuple};

use super::{
    AstNodeKind::{MacroCall, MacroDef},
    TokenKind::Comma,
    *,
};

impl GazmParser {
    pub fn parse_macro_call(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (label, args))) = ms(pair(
            get_label_string,
            parse_bracketed(Self::parse_expr_list0),
        ))(input)?;

        let node = from_item_children_tspan(MacroCall(label), &args, sp);
        Ok((rest, node))
    }

    pub fn parse_macro_def(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (label, args, body))) = ms(preceded(
            CommandKind::Macro,
            tuple((
                get_label_string,
                parse_macrodef_args,
                parse_block(many0(Self::parse_next_source_chunk)),
            )),
        ))(input)?;

        let body: Vec<Node> = body.into_iter().flatten().collect();

        let node = from_item_children_tspan(MacroDef(label, args.into()), &body, sp);
        Ok((rest, node))
    }
}

pub fn is_parsing_macro_def(i: TSpan) -> bool {
    i.extra().is_parsing_macro_def
}

pub fn set_parsing_macro(i: TSpan, v: bool) -> TSpan {
    i.lift_extra(|e| ParseContext {
        is_parsing_macro_def: v,
        ..e
    })
}

fn parse_macrodef_args(input: TSpan) -> PResult<Vec<String>> {
    parse_bracketed(sep_list0(get_label_string, Comma))(input)
}
