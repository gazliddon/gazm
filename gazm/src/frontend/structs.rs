#![deny(unused_imports)]


use super::{
    get_text,
    parse_expr,
    AstNodeKind, Node,
    StructMemberType,
    parse_block,  CommandKind, GazmParser, PResult, TSpan,
    TokenKind::{CloseSquareBracket, Colon, Comma, OpenSquareBracket, Label},
    from_item_tspan,
    from_item_kids_tspan,
    from_item_kid_tspan,
};

use unraveler::{map, match_span as ms, opt, pair, preceded, sep_list0, succeeded, tag, tuple};

use CommandKind::Struct;

impl GazmParser
{
    pub fn parse_array_def(input: TSpan) -> PResult<Node> {
        let (rest, (_, matched, _)) =
            tuple((OpenSquareBracket, parse_expr, CloseSquareBracket))(input)?;

        Ok((rest, matched))
    }

    pub fn parse_struct_entry(input: TSpan) -> PResult<Node> {
        let (rest, (name, _, (entry_span, entry_type), (array_def_sp, array))) = tuple((
            Label,
            Colon,
            parse_struct_arg_type,
            ms(opt(Self::parse_array_def)),
        ))(input)?;

        let size =entry_type.to_size_item();

        let kids = [
            array.unwrap_or(Self::from_num_tspan(1, array_def_sp)),
            from_item_tspan(AstNodeKind::Mul, array_def_sp),
            from_item_tspan(size, entry_span),
        ];

        let name = get_text(name).to_owned();

        let expr = from_item_kids_tspan(AstNodeKind::Expr, &kids, entry_span);
        let node = from_item_kid_tspan(AstNodeKind::StructEntry(name), expr, input);

        Ok((rest, node))
    }

    pub fn parse_struct(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (label, entries))) = ms(pair(
            preceded(Struct, Label),
            parse_block(succeeded(
                sep_list0(Self::parse_struct_entry, Comma),
                opt(Comma),
            )),
        ))(input)?;

        let text = get_text(label);

        let node = from_item_kids_tspan(AstNodeKind::StructDef(text), &entries, sp);

        Ok((rest, node))
    }
}

fn parse_struct_arg_type(input: TSpan) -> PResult<(TSpan, StructMemberType)> {
    let as_arg_type =
        |i| -> StructMemberType { get_text(i).to_string().parse::<StructMemberType>().unwrap() };
    ms(map(tag(Label), as_arg_type))(input)
}

// Always compile so I get IDE errors
#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::{frontend::*, opts::Opts};

    use grl_eval::OperatorTraits;
    // Only include dev deps if test cfg
    #[cfg(test)]
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    use grl_sources::SourceFile;
    use thin_vec::thin_vec;
    use unraveler::Collection;
    use LabelDefinition::Text;

    #[test]
    fn test_struct() {
        use AstNodeKind::*;

        let text = r#"
        struct my_struct 
        { test : byte, spanner : byte,
        book  : word
        }"#;

        let opts = Opts::default();

        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);

        let ts: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", ts);

        let span = make_tspan(&tokens, &sf, &opts);

        let (rest, matched) = parse_struct(span).unwrap();

        let sub_kinds: Vec<_> = matched.children.iter().map(|t| &t.item).collect();
        println!("Kids: {:?}", sub_kinds);

        let items = get_items(&matched);
        let desired = (
            StructDef("my_struct".to_owned()),
            thin_vec![
                StructEntry("test".into()),
                StructEntry("spanner".into()),
                StructEntry("book".into()),
            ],
        );

        assert_eq!(items, desired);

        assert!(rest.is_empty());
    }
}
