use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server, };
use crate::ctx::Opts;
use log::info;
use super::backend::Backend;

async fn do_lsp_2(opts: Opts) {
    use log::LevelFilter;

    simple_logging::log_to_file("/Users/garyliddon/development/gazm/glsp/log.log", LevelFilter::Info).unwrap();
    info!("Starting up gazm lsp");

    // tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| Backend::new(client,opts));

    Server::new(stdin, stdout, socket).serve(service).await;
}


