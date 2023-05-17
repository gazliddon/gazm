use super::backend::Backend;
use crate::opts::Opts;
use log::info;
use log::LevelFilter;
use serde::Deserialize;
use std::path::PathBuf;
use tower_lsp::{LspService, Server};

#[derive(Default,Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct LspConfig {
    pub log_file: Option<PathBuf>,
}

pub fn do_lsp(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: have the logger log to the file in the lsp_config opt
    if let Some(log_path) = &opts.lsp_config.log_file {
        simple_logging::log_to_file(log_path, LevelFilter::Info).unwrap();
    }

    info!("**Starting up gazm lsp");

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| Backend::new(client, opts));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            info!("**About to create server");
            let server = Server::new(stdin, stdout, socket);
            server.serve(service).await;
            info!("**About to exit");
        });

    Ok(())
}
