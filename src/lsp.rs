use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use silex_lsp::syntax::{generate_diagnostics, tokenize_document};

use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SilexLanguageServer {
    pub client: Client,
    pub documents: Mutex<HashMap<Url, String>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for SilexLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                inlay_hint_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    SemanticTokensRegistrationOptions  {
                        text_document_registration_options: {
                            TextDocumentRegistrationOptions {
                                document_selector: Some(vec![DocumentFilter {
                                    language: Some("silex".to_string()),
                                    scheme: Some("file".to_string()),
                                    pattern: None,
                                }]),
                            }
                        },
                        semantic_tokens_options: SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    "keyword".into(),
                                    "function".into(),
                                    "controlFlow".into(),
                                    "literal".into(),
                                    "variable".into(),
                                    "operator".into(),
                                ],
                                token_modifiers: vec![],
                            },
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(true),
                        },
                        static_registration_options: StaticRegistrationOptions::default(),
                    }
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Silex Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client.log_message(MessageType::INFO, "Shutting down server").await;
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
    
        // Retrieve the document content
        let documents = self.documents.lock().await;
        if let Some(content) = documents.get(&uri) {
            // Process the content for hover information
            let word = "example"; // Replace with actual logic to find the word at the hover position
            let contents = HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Silex Info**\nYou hovered over: `{}`", word),
            });
    
            Ok(Some(Hover {
                contents,
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
    
        self.client
            .log_message(MessageType::INFO, "Silex document opened")
            .await;

        // // Store the document content
        // self.documents.lock().await.insert(uri.clone(), text.clone());

        // // Generate diagnostics
        // let diagnostics = generate_diagnostics(&text);
        // self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    
        // // Trigger semantic tokens update
        // if let Ok(Some(tokens)) = self.semantic_tokens_full(SemanticTokensParams { 
        //   text_document: TextDocumentIdentifier { uri: uri.clone() },
        //   partial_result_params: Default::default(), // Optional field for partial results
        //   work_done_progress_params: Default::default(), // Optional field for work done progress
        // }).await {
        //     self.client.log_message(MessageType::INFO, format!("Semantic tokens generated for {}", uri)).await;
        // }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.content_changes.first().unwrap().text.clone();
    
        self.client
            .log_message(MessageType::INFO, "Silex document changed")
            .await;

        // // Update the document content
        // self.documents.lock().await.insert(uri.clone(), text.clone());
        
        // // Generate diagnostics
        // let diagnostics = generate_diagnostics(&text);
        // self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    
        // // Trigger semantic tokens update
        // if let Ok(Some(tokens)) = self.semantic_tokens_full(SemanticTokensParams { 
        //   text_document: TextDocumentIdentifier { uri: uri.clone() },
        //   partial_result_params: Default::default(), // Optional field for partial results
        //   work_done_progress_params: Default::default(), // Optional field for work done progress
        // }).await {
        //     self.client.log_message(MessageType::INFO, format!("Semantic tokens updated for {}", uri)).await;
        // }
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        // Implement your logic to handle the semantic tokens and return them.
        let tokens = tokenize_document(&params.text_document.uri.to_string()); // Assuming this function exists

        let result = SemanticTokens {
            result_id: None,
            data: tokens.unwrap().data,
        };

        Ok(Some(SemanticTokensResult::Tokens(result)))
    }
}
