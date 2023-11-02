use gazm::{
    cli::{parse_command_line, styling::get_banner},
    fmt, frontend,
    info_mess, lsp, messages,
    opts::{BuildType, Opts},
    status_mess,
    assembler::Assembler,
};

fn do_build(opts: &Opts) -> Result<(), Box<dyn std::error::Error>> {
    let mess = messages::messages();
    mess.set_verbosity(&opts.verbose);

    if let Some(assemble_dir) = &opts.assemble_dir {
        std::env::set_current_dir(assemble_dir)?;
    }

    let mut asm = Assembler::new(opts.clone());

    match opts.build_type {
        BuildType::Test => {
            status_mess!("Testing! {}", opts.project_file.to_string_lossy());
            frontend::test(&opts.project_file);
            status_mess!("Done!");
        }

        BuildType::Format => {
            status_mess!("Format file");
            fmt::fmt(opts)?;
        }

        BuildType::Lsp => {
            status_mess!("LSP");
            lsp::do_lsp(opts)?;
        }

        // Build of check to see if build is okay
        BuildType::Build | BuildType::Check => {
            status_mess!("{}", get_banner());
            mess.indent();
            status_mess!("Verbosity: {:?}", &opts.verbose);

            if opts.no_async {
                status_mess!("Async: NO ASYNC");
            }

            asm.assemble()?;

            // Only write outputs if this is of buildtype Build
            if opts.build_type == BuildType::Build {
                asm.write_outputs()?;
            }

            mess.deindent();
            info_mess!("")
        }
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env::{current_dir, set_current_dir};
    let matches = parse_command_line();
    let opts = Opts::from_arg_matches(matches)?;

    // Todo move directory handling into assemble_from_opts
    // probably as a function of Opts
    let cur_dir = current_dir().unwrap();

    let ret = do_build(&opts);

    if let Err(ref e) = ret {
        println!("{e}")
    }

    set_current_dir(cur_dir)?;

    Ok(())
}

#[cfg(test)]
mod test {
    // use crate::Assembler;
    use std::path::PathBuf;

    use super::*;
    // use pretty_assertions::*;

    // use gazm::ctx::Context;

    fn make_opts(file_name: &str) -> Opts {
        let mut ret = Opts::default();
        ret.project_file = PathBuf::from(file_name);
        ret.build_type = BuildType::Check;
        ret
    }

    #[test]
    fn test_circ() {
        let opts = make_opts("assets/test_src/circular_inc.gazm");
        let mut asm = Assembler::new(opts.clone());
        let res = asm.assemble();
        println!("{res:#?}");
        // assert!(res.is_ok());
    }
}
