use emu::utils::sources::{TextEdit, TextEditTrait, TextPos};

use super::ctx::Opts;

use crate::ctx::Context;
use crate::error::GResult;
use crate::gazm::{create_ctx, reassemble_ctx, with_state};

use std::path::PathBuf;
use std::time::Instant;

pub fn do_lsp(opts: Opts) -> GResult<()> {
    let arc_ctx = create_ctx(opts);

    let start = Instant::now();
    let res = reassemble_ctx(&arc_ctx);
    let dur = start.elapsed().as_secs_f64();
    println!("assemble time took {:?}", dur);

    println!("{:?}", res);

    with_state(&arc_ctx, |ctx| {
        let p = PathBuf::from("/Users/garyliddon/development/stargate/src/stargate.src");

        ctx.with_source_file(&p, |source| {
            println!("Before edit");
            for line in 0..5 {
                println!("{}", source.get_line(line).unwrap());
            }
            println!("");
        });

        ctx.edit_source_file(&p, |source| {
            // start == end means an insert of a line
            let start = TextPos {
                line: 0,
                character: 0,
            };

            let end = start.clone();

            let text = "; An Extra Comment\n";

            let te = TextEdit::from_pos(start, end, text);

            source.edit(&te).unwrap();
        });



        ctx.with_source_file(&p, |source| {
            println!("After Edit");
            for line in 0..5 {
                println!("{}", source.get_line(line).unwrap());
            }
            println!("");
        });
    });

    let start = Instant::now();

    let res = reassemble_ctx(&arc_ctx)?;
    let dur = start.elapsed().as_secs_f64();

    println!("reassemble time took {:?}", dur);

    println!("{:?}", res);

    Ok(())
}
