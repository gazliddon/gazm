use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server, };
use crate::ctx::Opts;
use log::info;
use super::backend::Backend;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct LspConfig {
    pub log_file: Option<PathBuf>,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self { log_file: Default::default() }
    }
}

async fn run_lsp(opts: Opts) {
    use log::LevelFilter;

    // TODO
    // have the logger log to the file in the lsp_config opt
    if let Some(log_path) = &opts.lsp_config.log_file {
        simple_logging::log_to_file(log_path, LevelFilter::Info).unwrap();
    }

    info!("Starting up gazm lsp");

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| Backend::new(client,opts));

    Server::new(stdin, stdout, socket).serve(service).await;
}

pub fn do_lsp(opts: Opts)  -> Result<(), Box<dyn std::error::Error>> {
    use tokio::runtime::Runtime;
    let rt  = Runtime::new()?;
    rt.block_on(run_lsp(opts));
    Ok(())
}

