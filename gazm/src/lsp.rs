use emu::utils::sources::{TextPos, TextEdit };

use super::ctx::Opts;

use crate::ctx::Context;
use crate::error::GResult;
use crate::gazm::{create_ctx, reassemble_ctx, with_state};

use std::path::PathBuf;
use std::time::Instant;

pub fn do_lsp(opts: Opts) -> GResult<()> {
    let ctx = create_ctx(opts);

    let start = Instant::now();
    let res = reassemble_ctx(&ctx);
    let dur = start.elapsed().as_secs_f64();
    println!("assemble time took {:?}", dur );

    println!("{:?}", res);

    with_state(&ctx, |_ctx| {
        let p = PathBuf::from("/Users/garyliddon/development/stargate/src/stargate.src");

        let source  = _ctx.sources_mut().get_source_mut(&p).unwrap();

        for line in 0..5 {
            println!("{:?}", source.get_line(line)); 
        }


        // start == end means an insert of a line
        let start = TextPos { line : 0, character: 0 };
        let end = start.clone();
        
        let text = "; An Extra Comment\n";

        let te = TextEdit::new(start, end, text);

        source.edit(&te).unwrap();

        for line in 0..5 {
            println!("{:?}", source.get_line(line)); 
        }

        _ctx.get_token_store_mut().invalidate_tokens(&p);
    });

    let start = Instant::now();

    let res = reassemble_ctx(&ctx)?;
    let dur = start.elapsed().as_secs_f64();
    println!("reassemble time took {:?}", dur );

    println!("{:?}", res);

    Ok(())
}
