use gazm::{
    assembler::Assembler,
    cli::{parse_command_line, styling::get_banner},
    error::{ErrorCollectorTrait, GazmErrorKind},
    frontend, info_mess, messages,
    opts::{BuildType, Opts},
    status_mess,
    cpu6809::assembler::Compiler6809,
};

fn do_build(opts: &Opts) -> Result<(), GazmErrorKind> {
    let mess = messages::messages();
    mess.set_verbosity(&opts.verbose);

    if let Some(assemble_dir) = &opts.assemble_dir {
        std::env::set_current_dir(assemble_dir).expect("Can't change dir")
    }

    let mut asm = Assembler::<Compiler6809>::new(opts.clone());

    match opts.build_type {
        BuildType::Test => {
            status_mess!("Testing! {}", opts.project_file.to_string_lossy());
            frontend::test_it(opts);
            status_mess!("Done!");
        }

        BuildType::Format => {
            status_mess!("Format file");
            todo!()
        }

        BuildType::Lsp => {
            status_mess!("LSP");
            todo!()
            // lsp::do_lsp(opts)?;
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

    match ret {
        Err(GazmErrorKind::UserErrors(user_errors)) => {
            for e in user_errors.to_vec() {
                e.as_ref().print_pretty(opts.verbose_errors)
            }
        }

        Err(e) => {
            println!("{e}");
        }

        Ok(..) => {}
    };

    set_current_dir(cur_dir)?;

    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    // use crate::Assembler;
    use std::path::PathBuf;

    use super::*;

    fn make_opts(file_name: &str) -> Opts {
        let mut ret = Opts::default();
        ret.project_file = PathBuf::from(file_name);
        ret.build_type = BuildType::Check;
        ret
    }

    // TODO Reinstate this test and make circular includes error
    // #[test]
    fn test_circ() {
        let opts = make_opts("assets/test_src/circular_inc.gazm");
        let mut asm = Assembler::new(opts.clone());
        let res = asm.assemble();
        assert!(res.is_ok());
    }
}
