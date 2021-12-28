use crate::{cli, commands, comments, expr, fileloader, item, labels, locate::Position, opcodes, util};

use nom::{
    branch::alt,
    bytes::complete::{ take_until, is_not },
    character::complete::{ multispace1, multispace0, line_ending, }, combinator::{opt, all_consuming, eof, not, recognize},  multi::{ many0, many1 }, sequence::{ pair, terminated, preceded }};

use crate::error::{IResult, ParseError};
use crate::locate::{ Span, AsSpan };
use crate::item::Node;

fn get_line(input : Span)-> IResult<Span> {
        let (rest, line) =
            preceded(multispace0, 
                     terminated(recognize(many0(is_not("\n"))), opt(line_ending)))(input)?;

        Ok((rest,line))
}

pub fn tokenize_str<'a>(input : Span<'a>) -> Result<Vec<item::Node>, ParseError> {

    use item::{ Item::*, Node };
    use commands::parse_command;
    use labels::parse_label;
    use opcodes::parse_opcode;
    use util::parse_assignment;
    use util::ws;

    let mut source = input.clone();

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
        let (rest, line) = get_line(source)?;

        source = rest;

        if !line.is_empty() {

            let (input, comment) = comments::strip_comments(line)?;
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

            let (_, (label,body)) = all_consuming(pair(opt(parse_label),body))(input)?;
            let label = label.map(mk_pc_equate);
            push_some(&label);
            push_some(&Some(body));
        }
    }

    Ok(items)
}

use std::path::Path;

struct SoureFile {
    text : String,
}

pub fn tokenize_file<P: AsRef<Path>>(fl : &fileloader::FileLoader, file_name : P) -> Result<Node, ParseError> {
    use item::Item::*;

    let file_name = file_name.as_ref().to_path_buf();


    println!("Tokenizing: {:?}", file_name.as_path());

    let (loaded_name,source) = fl.read_to_string(&file_name)
        .map_err(|e| ParseError::from_text(&e.to_string()))?;

    let pos = Position::from_usize((0,source.len()));

    let input = source.as_span();

    let mut matched = tokenize_str(input)?;

    for tok in &mut matched {
        if let Include(file) = tok.item() {
            let inc_source = tokenize_file(fl, file.clone())?;
            *tok = inc_source;
        }
    }

    let ret = Node::from_item(File(loaded_name))
        .with_children(matched)
        .with_ctx(pos);

    Ok(ret)
}


pub fn tokenize( ctx : &cli::Context ) -> Result<Node, ParseError> {
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
