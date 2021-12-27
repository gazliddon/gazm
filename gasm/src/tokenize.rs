use crate::{ cli,item,commands, comments, labels, expr,  opcodes, fileloader, util };

use nom::{
    character::complete::{ multispace1, multispace0, line_ending, },
    bytes::complete::{ take_until, is_not },
    sequence::{ pair, terminated, preceded },
    combinator::{opt, all_consuming, eof, not, recognize},
    branch::alt,
    multi::{ many0, many1 }
};

use crate::error::{IResult, ParseError};
use crate::locate::Span;

pub fn tokenize_str(source : Span) -> IResult<Vec<item::Node>> {
    use item::{ Item::*, Node };
    use comments::strip_comments_and_ws;
    use commands::parse_command;
    use labels::parse_label;
    use opcodes::parse_opcode;
    use util::parse_assignment;


    use util::ws;

    let mut source = source;

    let mut items : Vec<Node> = vec![];

    let mut push_some = |x : &Option<Node> | {
        if let Some(x) = x {
            items.push(x.clone())
        }
    };

    let mk_pc_equate = |node : Node| {
        let pos = node.ctx().clone();
        let children = vec![node, Node::from_item(Pc).with_ctx(pos.clone())];
        Node::from_item(Assignment).with_children(children).with_ctx(pos)
    };

    while !source.is_empty() {

        let (rest, line) =
            preceded(multispace0, 
            terminated(recognize(many0(is_not("\n"))), opt(line_ending))
                                    )(source)?;

        source = rest;

        if !line.is_empty() {

            let (input, comment) = strip_comments_and_ws(line.into())?;

            push_some(&comment);

            if input.is_empty() {
                continue;
            }

            if let Ok((_,equate )) = all_consuming(ws::<_,_,ParseError>(parse_assignment))(input) {
                push_some(&Some(equate));
                continue;
            }

            if let Ok((_,label)) = all_consuming(ws::<_,_,ParseError>(labels::parse_label))(input) {
                let node = mk_pc_equate(label);
                push_some(&Some(node));
                continue;
            }

            let body = alt(( ws::<_,_,ParseError>( parse_opcode ),ws::<_,_,ParseError>( parse_command ) ));

            let res = all_consuming(pair(opt(parse_label),body))(input);

            if let Ok((_, (label,body))) = res {
                let label = label.map(mk_pc_equate);
                push_some(&label);
                push_some(&Some(body));
            } else {
                println!("{:?}", res);
                println!("Input: {:?}", input);
                panic!()
            }
        }

    }

    Ok((source, items))
}

use std::path::Path;

pub fn tokenize_file<P: AsRef<Path>>(fl : &fileloader::FileLoader, file_name : P) -> Result<item::Node, Box<dyn std::error::Error>> {
    use item::Item::*;
    let file_name = file_name.as_ref().to_path_buf();

    println!("Tokenizing: {:?}", file_name.as_path());

    let (loaded_name,source) = fl.read_to_string(&file_name)?;

    let source = Span::new_extra(&source, &source);

    let res = tokenize_str(source);

    match res {
        Ok((rest,mut matched)) => {

            for tok in &mut matched {
                if let Include(file) = tok.item() {
                    let inc_source = tokenize_file(fl, file.clone())?;
                    *tok = inc_source;
                }
            }
            use item::Node;

            let ret = Node::from_item(File(loaded_name))
                .with_children(matched)
                .with_pos(source,rest);

            Ok(ret)
        },

        Err(e) => {
            if let nom::Err::Error(pe) = e {
                println!("{:?}", pe.message);
            }
            panic!()
        },
    }
}


pub fn tokenize( ctx : &cli::Context ) -> Result<item::Node, Box<dyn std::error::Error>> {
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

}
