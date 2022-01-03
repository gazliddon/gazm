use crate::{cli, commands, comments, expr, fileloader, item, labels, locate::Position, messages, opcodes, util};

use nom::{AsBytes, branch::alt, bytes::complete::{ take_until, is_not }, character::complete::{ multispace1, multispace0, line_ending, }, combinator::{opt, all_consuming, eof, not, recognize}, multi::{ many0, many1 }, sequence::{ pair, terminated, preceded }};

use crate::error::{IResult, ParseError, UserError};
use crate::locate::{ Span, mk_span };
use crate::item::Node;

fn get_line(input : Span)-> IResult<Span> {
        let (rest, line) =
            preceded(multispace0, 
                     terminated(recognize(many0(is_not("\n"))), opt(line_ending)))(input)?;

        Ok((rest,line))
}

struct Tokens {
    text : String,
    tokens: Vec<item::Node>
}

pub fn tokenize_str<'a>(input : Span<'a>) -> Result<Node, ParseError<'a>> {

    use item::{ Item::*, Node };
    use commands::parse_command;
    use labels::parse_label;
    use opcodes::parse_opcode;
    use util::parse_assignment;
    use util::ws;

    let ret = Node::from_item(Block,input);

    let mut source = input.clone();

    let mut items : Vec<Node> = vec![];

    let mut push_some = |x : &Option<Node> | {
        if let Some(x) = x {
            items.push(x.clone())
        }
    };

    let mk_pc_equate = |node : Node| {
        let pos = node.ctx().clone();
    
        match &node.item {
            Label(name) => {
                Node::from_item(AssignmentFromPc(name.clone()), pos)

            },
            LocalLabel(name) => {
                Node::from_item(LocalAssignmentFromPc(name.clone()), pos)
            }

            _ => panic!("shouldn't happen")
        }
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

    Ok(ret.with_children(items))
}

use std::path::{Path, PathBuf};


extern crate colored;
use colored::*;

pub fn tokenize_file(depth: usize, _ctx : &cli::Context, fl : &fileloader::FileLoader, file : &std::path::PathBuf, parent : &std::path::PathBuf ) -> Result<Node, UserError> {
    use super::messages::*;
    use item::Item::*;
        let x = messages::messages();

    let (file_name, source) = fl.read_to_string(file.clone()).unwrap();

    let action = if depth == 0 {
        "Tokenizing"
    } else {
        "Tokenizing including"
    };

    let mapper = |e| UserError::from_parse_error(e, &file_name);

    let comp_msg = format!("{} {}", action, file_name.to_string_lossy());

    x.info(&comp_msg);

    let input = Span::new(&source);
    let mut matched = tokenize_str(input).map_err(mapper)?;
    matched.item = TokenizedFile(file.clone(),parent.clone(), source.clone());

    // Tokenize includes
    for n in matched.children.iter_mut() {
        if let Some(inc_file) = n.get_include_file() {
            x.indent();
            *n = tokenize_file(depth+1, _ctx, fl, &inc_file.to_path_buf(), file)?.into();
            x.deindent();
        }
    }

    Ok(matched)
}

pub fn tokenize( ctx : &cli::Context ) -> Result<Node, UserError> {
    use fileloader::FileLoader;

    let file = ctx.file.clone();

    let mut paths = vec![];

    if let Some(dir) = file.parent() {
        paths.push(dir);
    }

    let fl = FileLoader::from_search_paths(&paths);
    let parent = PathBuf::new();

    tokenize_file(0, ctx, &fl,&ctx.file, &parent)
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

#[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

}
