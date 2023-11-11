#![forbid(unused_imports)]
use crate::assembler::Assembler;

use crate::{astformat, error::GResult, gazmsymbols::Serializable, status_err, status_mess};

use grl_sources::{fileloader::FileIo, grl_utils::hash::get_hash, SourceDatabase};

use anyhow::Context as AnyContext;
use std::fs;
use std::path::Path;

fn join_paths<P: AsRef<Path>, I: Iterator<Item = P>>(i: I, sep: &str) -> String {
    let z: Vec<String> = i.map(|s| s.as_ref().to_string_lossy().into()).collect();
    z.join(sep)
}

impl Assembler {
    /// Write any outputs that need writing
    pub fn write_outputs(&mut self) -> GResult<()> {
        self.write_bin_chunks()?;
        self.checksum_report();
        self.write_lst_file()?;
        self.write_source_mapping()?;
        self.write_sym_file()?;
        self.write_deps_file()?;
        self.write_ast_file()?;
        Ok(())
    }

    fn write_bin_chunks(&mut self) -> GResult<()> {
        for bin_to_write in &self.asm_out.bin_to_write_chunks {
            let (addr, len, file) = bin_to_write.write_bin(&mut self.source_file_loader);
            status_mess!("Written binary: {:?} ${addr:x} ${len:x}", file);
        }

        Ok(())
    }

    fn write_file<P: AsRef<Path>>(&mut self, p: P, txt: &str) -> GResult<String> {
        let full_file_name = self.expand_path(p);
        fs::write(&full_file_name, txt)
            .with_context(|| format!("Unable to write {:?}", full_file_name))?;
        Ok(full_file_name.to_string_lossy().into_owned())
    }

    pub fn write_lst_file(&mut self) -> GResult<()> {
        if let Some(lst_file) = &self.opts.lst_file {
            let lst_file = self.get_vars().expand_vars(lst_file);

            let text = self.asm_out.lst_file.lines.join("\n");

            fs::write(&lst_file, text)
                .with_context(|| format!("Unable to write list file {lst_file}"))?;

            status_mess!("Written lst file {lst_file}");
        }

        Ok(())
    }

    pub fn write_ast_file(&mut self) -> GResult<()> {
        if let Some(ast_file) = &self.opts.ast_file {
            let ast_file = self.expand_path(ast_file);
            status_mess!("Writing ast: {:?}", ast_file);

            if let Some(ast) = &self.asm_out.ast {
                let x = astformat::as_string(ast.as_ref().root());
            fs::write(&ast_file, x)
                .with_context(|| format!("Unable to write list file {}",ast_file.to_string_lossy() ))?;
            } else {
                status_err!("No AST file to write");
            }
        }
        Ok(())
    }

    pub fn write_deps_file(&mut self) -> GResult<()> {
        if let Some(deps) = &self.opts.deps_file {
            if let Some(sym_file) = &self.opts.source_mapping {
                let sym_file = self.expand_path(sym_file);
                let sf = self.get_source_file_loader();
                let read = join_paths(sf.get_files_read().iter(), " \\\n");
                let written = join_paths(sf.get_files_written().iter(), " \\\n");
                let deps_line_2 = format!("{written} : {:?}", sym_file);
                let deps_line = format!("{deps_line_2}\n{:?} : {read}", sym_file);

                status_mess!("Writing deps file: {deps}");

                std::fs::write(deps, deps_line)
                    .with_context(|| format!("Unable to write {deps}"))?;
            }
        }

        Ok(())
    }

    pub fn write_sym_file(&mut self) -> GResult<()> {
        if let Some(syms_file) = &self.opts.syms_file {
            let syms_file = self.opts.vars.expand_vars(syms_file);
            println!("{:?}", self.opts.vars);
            let serialized: Serializable = self.get_symbols().into();
            let json_text = serde_json::to_string_pretty(&serialized).unwrap();
            let file_name = self.write_file(syms_file, &json_text)?;
            status_mess!("Writen symbols file: {}", file_name);
        }

        Ok(())
    }

    fn write_source_mapping(&mut self) -> GResult<()> {
        if let Some(sym_file) = &self.opts.source_mapping {
            let sym_file = self.get_vars().expand_vars(sym_file);
            let sd: SourceDatabase = (&*self).into();
            let file_name = sd
                .write_json(&sym_file)
                .with_context(|| format!("Unable to write {sym_file}"))?;

            status_mess!("Written source mappings {file_name}");
        }

        Ok(())
    }

    fn checksum_report(&self) {
        if !self.opts.checksums.is_empty() {
            let mess = crate::messages::messages();

            let mut errors = vec![];

            for (name, csum) in &self.opts.checksums {
                let data = self
                    .binary()
                    .get_bytes(csum.addr, csum.size)
                    .expect("Binary error");
                let this_hash = get_hash(data);
                let expected_hash = csum.sha1.to_lowercase();

                if this_hash != expected_hash {
                    let hash = format!("{name} : {this_hash} != {expected_hash}");
                    errors.push(hash);
                }
            }

            if errors.is_empty() {
                status_mess!("✅: {} Checksums correct", self.opts.checksums.len())
            } else {
                mess.error("❌ : Mismatched Checksums");
                mess.indent();
                for name in errors {
                    status_err!("{name} : ❌");
                }
                mess.deindent();
            }
        }
    }
}
