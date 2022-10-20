
#![allow(dead_code)]
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server, };
use log::info;

use serde_json::Value;
use tower_lsp::jsonrpc::Result as TResult;

use emu::utils::sources::{TextEdit, TextEditTrait, TextPos};

use super::ctx::Opts;

use crate::ctx::Context;
use crate::error::GResult;
use crate::gazm::{create_ctx, reassemble_ctx, with_state};

use std::path::PathBuf;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;

use std::sync::{Arc,Mutex};

#[derive(Debug)]
struct Backend {
    client: Client,
    ctx: Arc<Mutex<Context>>,
}
pub fn do_lsp_23(opts: Opts) -> Result<(), Box<dyn std::error::Error>> { 

    let _arc_ctx = create_ctx(opts);

    let rt  = Runtime::new()?;

// Spawn the root task
    rt.block_on(async {
    });

    Ok(())
}

async fn do_lsp_2(opts: Opts) {
    let ctx = create_ctx(opts);
    use log::LevelFilter;

    simple_logging::log_to_file("/Users/garyliddon/development/gazm/glsp/log.log", LevelFilter::Info).unwrap();

    info!("Starting up gazm lsp");

    // tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend { client, ctx });

    Server::new(stdin, stdout, socket).serve(service).await;
}

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
            let text = "; An Extra Comment\n";
            let te = TextEdit::new(0,0, 0,0, text);
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


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> TResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),

                hover_provider: Some(HoverProviderCapability::Simple(true) ),

                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("initialized");
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> TResult<()> {
        info!("shutdown");
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        info!("did_change_workspace_folders");
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        info!("did_change_configuration");
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        info!("did_change_watched_files");
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> TResult<Option<Value>> {
        info!("execute_command");
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        info!("did_open");
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        info!("did_change");
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        info!("did_save");
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        info!("did_close");
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> TResult<Option<CompletionResponse>> {
        info!("completion");
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, params: HoverParams) -> TResult<Option<Hover>> {
        info!("hover");
        info!("{}", params.text_document_position_params.text_document.uri.path());

        let x = vec![];

        let xx = self.client.configuration(x).await;

        if let Ok(xx) = xx {
                for f in xx {
                    info!("item: {}", f)
                }
        }

        let _ = params;

        let reply = Some(Hover {
            contents: HoverContents::Markup(MarkupContent{
                kind: MarkupKind::Markdown,
                value: "# You Are A\nWanker".to_string(),
            }),
            range: None,
        });

        Ok(reply)
    }
}
