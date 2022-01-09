use crate::{cli, commands, comments, expr, fileloader, item, labels, locate::Position, messages, opcodes, util};

use nom::{AsBytes, branch::alt, bytes::complete::{ take_until, is_not }, character::complete::{ multispace1, multispace0, line_ending, }, combinator::{opt, all_consuming, eof, not, recognize}, multi::{ many0, many1 }, sequence::{ pair, terminated, preceded }};

use crate::error::{IResult, ParseError, UserError};
use crate::locate::Span;
use crate::item::{ Node, Item };

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

pub fn tokenize_file_from_str<'a>(file : &str,input : &'a str) -> Result<Node, ParseError<'a>> { 
    let input = Span::new(input);
    let source = input.to_string();
    let mut matched = tokenize_str(input)?;
    matched.item = Item::TokenizedFile(file.into(), file.into(), source);
    Ok(matched)
}

pub fn tokenize_str(input : Span<'_>) -> Result<Node, ParseError> {

    use item::{ Item::*, Node };
    use commands::parse_command;
    use labels::parse_label;
    use opcodes::parse_opcode;
    use util::parse_assignment;
    use util::ws;

    let ret = Node::from_item(Block,input);

    let mut source = input;

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

pub fn tokenize_file(depth: usize, _ctx : &cli::Context, fl : &fileloader::FileLoader, file : &std::path::Path, parent : &std::path::Path ) -> Result<Node, UserError> {
    use super::messages::*;
    use item::Item::*;
        let x = messages::messages();

    let (file_name, source) = fl.read_to_string(file).unwrap();

    let action = if depth == 0 {
        "Tokenizing"
    } else {
        "Including"
    };

    let mapper = |e| UserError::from_parse_error(e, &file_name);

    let comp_msg = format!("{} {}", action, file_name.to_string_lossy());

    x.info(&comp_msg);

    let input = Span::new(&source);
    let mut matched = tokenize_str(input).map_err(mapper)?;
    matched.item = TokenizedFile(file.to_path_buf(),parent.to_path_buf(), source.clone());

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
