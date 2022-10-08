
use super::ctx::Opts;

use crate::ctx::Context;
use crate::error::GResult;
use crate::gazm::{ create_ctx,reassemble_ctx, with_state };

pub fn do_lsp(opts : Opts) -> GResult<()>{

    let ctx = create_ctx(opts);

    let res = reassemble_ctx(&ctx);

    println!("{:?}", res);

    with_state(&ctx, |_ctx| {
        // edit file here
    });

    let res = reassemble_ctx(&ctx)?;
    println!("{:?}", res);

    Ok(())
}
