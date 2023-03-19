use crate::ctx::{Context, Opts};
use crate::error::GResult;
use crate::gazm::{create_ctx, reassemble_ctx, with_state, Assembler};
use emu::utils::sources::TextEdit;
use log::{ info, error };
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result as TResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tower_lsp::jsonrpc;
use request::{
    GotoDeclarationParams, GotoDeclarationResponse, GotoImplementationParams,
    GotoImplementationResponse, GotoTypeDefinitionParams, GotoTypeDefinitionResponse,
};

pub struct Backend {
    pub client: Client,
    pub asm: Assembler,
}

pub fn to_text_edit<'a>(range: &Range, txt: &'a str) -> TextEdit<'a> {
    let te = TextEdit::new(
        range.start.line as usize,
        range.start.character as usize,
        range.end.line as usize,
        range.end.character as usize,
        txt,
    );
    te
}

impl Backend {
    pub fn new(client: Client, opts: Opts) -> Self {
        info!("Backend created!");
        let asm = Assembler::new(opts);
        Self { client, asm }
    }

    pub fn get_ast(_l : Location) -> Option<String> {
        None
    }
}

impl Assembler {
    fn apply_changes<P: AsRef<Path>>(
        &self,
        doc: P,
        content_changes: &Vec<TextDocumentContentChangeEvent>,
    ) -> GResult<()> {

        info!("Trying to apply changes to {}", doc.as_ref().to_string_lossy());

        // Apply the changes and abort on any errors
        for change in content_changes {
            if let Some(range) = change.range {
                let te = to_text_edit(&range, &change.text);
                info!("About to apply {:#?}", te);
                self.edit_file(&doc, |text_file| text_file.edit(&te))?;
            }
        }

        Ok(())
    }
}


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _init: InitializeParams) -> TResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo { name: "gazm".into(), version: None }),

            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),

                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),

                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
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

    async fn initialized(&self, _init: InitializedParams) {
        info!("initialized! yeah!");
        info!("{:#?}", _init);
        let x = self.asm.assemble();

        info!("Assembler results {:#?}", x);
        self.client
            .log_message(MessageType::INFO, "initialized it all!")
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

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> jsonrpc::Result<Option<GotoDeclarationResponse>> {
        info!("Goto declaration");
        let _ = params;
        error!("Got a textDocument/declaration request, but it is not implemented you twat");
        Err(jsonrpc::Error::method_not_found())
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
        ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let _ = params;
        error!("Got a textDocument/definition request, but it is not implemented you twat");
        Err(jsonrpc::Error::method_not_found())
    }


    // async fn goto_declaration(
    //     &self,
    //     params: GotoDeclarationParams,
    // ) -> jsonrpc::Result<Option<GotoDeclarationResponse>> {
    //     info!("Goto declaration");

    //     let range = Range {
    //         start: params.text_document_position_params.position.clone(),
    //         end : params.text_document_position_params.position.clone(),
    //     };

    //     let uri = params.text_document_position_params.text_document.uri;
    //     let l = Location { uri,  range};
    //     let resp = GotoDeclarationResponse::Scalar(l);
    //     Ok(Some(resp))
    // }

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

    async fn did_change(&self, x: DidChangeTextDocumentParams) {
        if x.text_document.uri.scheme() == "file" {
            let doc = x.text_document.uri.path();

            let e = self.asm.apply_changes(doc, &x.content_changes);

            match e {
                Err(e) => info!("Error {e}"),
                Ok(_) => info!("Applied changes"),
            };
        }

        let r = self.asm.reassemble();
        info!("ASM {:?}", r);

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
        info!(
            "{}",
            params
            .text_document_position_params
            .text_document
            .uri
            .path()
            );

        info!("{:#?}", params);



        let x = vec![];

        let xx = self.client.configuration(x).await;

        if let Ok(xx) = xx {
            for f in xx {
                info!("item: {}", f)
            }
        }

        let _ = params;

        let reply = Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "# You Are A\nWanker".to_string(),
            }),
            range: None,
        });

        Ok(reply)
    }
}
