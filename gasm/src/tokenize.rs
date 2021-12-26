use nom::IResult;
use nom::character::complete::{
    line_ending, multispace0, multispace1
};
use nom::sequence::{pair, tuple};
use nom::bytes::complete::tag_no_case;
use nom::combinator::{eof, opt, all_consuming};
use nom::branch::alt;

use crate::{ commands, util, comments, labels, expr, main, opcodes, fileloader };

use comments::strip_comments_and_ws;
use super::item::{ Item, Node };
use crate::Context;

use std::path::Path;

fn parse_assignment(input: &str) -> IResult<&str, Node> {
    use labels::parse_label;
    let (rest, (label, _, _, _, arg)) = tuple((
            parse_label,
            multispace1,
            tag_no_case("equ"),
            multispace1,
            expr::parse_expr
            ))(input)?;

    let ret = Node::from_item(Item::Assignment).with_children(vec![label, arg]);

    Ok((rest, ret))
}

pub fn tokenize_str(source : &str) -> IResult<& str, Vec<Node>> {
    use commands::parse_command;
    use labels::parse_label;
    use opcodes::parse_opcode;

    let mut items : Vec<Node> = vec![];

    let mut push_some = |x : &Option<Node> | {
        if let Some(x) = x {
            items.push(x.clone())
        }
    };

    use util::ws;

    let mk_pc_equate = |node : Node| {
        let children = vec![node, Node::from_item(Item::Pc)];
        Node::from_item(Item::Assignment).with_children(children)
    };

    for line in source.lines() {

        let (input, comment) = strip_comments_and_ws(line)?;
        push_some(&comment);

        if input.is_empty() {
            continue;
        }

        if let Ok((_,equate )) = all_consuming(ws(parse_assignment))(input) {
            push_some(&Some(equate));
            continue;
        }

        if let Ok((_,label)) = all_consuming(ws(labels::parse_label))(input) {
            let node = mk_pc_equate(label);
            push_some(&Some(node));
            continue;
        }

        let body = alt(( ws( parse_opcode ),ws( parse_command ) ));

        let res = all_consuming( pair(opt(parse_label),body))(input);

        if let Ok((_, (label,body))) = res {
            let label = label.map(mk_pc_equate);
            push_some(&label);
            push_some(&Some(body));
        } else {
            println!("{:?}", res);
            println!("Input: {:?}", input);
        }
    }

    // filter out empty comments
    let items = items
        .into_iter()
        .filter(|n| !n.is_empty_comment())
        .collect();

    Ok(("", items))
}

pub fn tokenize_file<P: AsRef<Path>>(fl : &fileloader::FileLoader, file_name : P) -> Result<Node, Box<dyn std::error::Error>> {
    let file_name = file_name.as_ref().to_path_buf();

    println!("Tokenizing: {:?}", file_name.as_path());

    let (loaded_name,source) = fl.read_to_string(&file_name)?;

    let (_rest, mut matched) = tokenize_str(&source).unwrap();

    for tok in &mut matched {
        if let Item::Include(file) = tok.item() {
            let inc_source = tokenize_file(fl, file.clone())?;
            *tok = inc_source;
        }
    }

    let ret = Node::from_item(Item::File(loaded_name)).with_children(matched);

    Ok(ret)
}


pub fn tokenize( ctx : &Context ) -> Result<Node, Box<dyn std::error::Error>> {
    use fileloader::FileLoader;

    let file = ctx.file.clone();

    let mut paths = vec![];

    if let Some(dir) = file.parent() {
        paths.push(dir);
    }

    let fl = FileLoader::from_search_paths(&paths);
    tokenize_file(&fl, file)
}


////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

#[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

    #[test]
    fn test_assignment() {
        let input = "hello equ $1000";
        let res = parse_assignment(input);
        assert!(res.is_ok());

        let (rest, matched) = res.unwrap();

        let args : Vec<_> = vec![
            Node::from_item(Item::Label("hello".to_string())),
            Node::from_number(4096)
        ];

        let desired = Node::from_item(Item::Assignment).with_children(args);

        assert_eq!(desired, matched);
        assert_eq!(rest, "");
    }
}
