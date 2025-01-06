use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use silex_lsp::syntax::{tokenize_document};

use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;

#[derive(Debug)]
pub struct SilexLanguageServer {
    pub client: Client,
    pub documents: Mutex<HashMap<Url, String>>,
    pub tab_size: AtomicU8,
}

#[tower_lsp::async_trait]
impl LanguageServer for SilexLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(init_options) = params.initialization_options {
            if let Some(tab_size) = init_options
                .get("tabSize")
                .and_then(|v| Some(v.as_u64()? as u8))
            {
                self.tab_size.store(tab_size as u8, std::sync::atomic::Ordering::Relaxed);
            }
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    SemanticTokensRegistrationOptions {
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
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::new("literal"),
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::COMMENT,
                                ],
                                token_modifiers: vec![],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        },
                        static_registration_options: StaticRegistrationOptions::default(),
                    }
                )),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL
                )),
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
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
    
        self.documents.lock().await.insert(uri.clone(), text);
        self.client
            .log_message(MessageType::INFO, format!("Document opened: {}", uri))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.content_changes[0].text.clone();
    
        self.documents.lock().await.insert(uri.clone(), text);
        self.client
            .log_message(MessageType::INFO, format!("Document changed: {}", uri))
            .await;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Some(settings) = params.settings.as_object() {
            if let Some(tab_size) = settings.get("edit.tabSize").and_then(|v| v.as_u64()) {
                self.client
                    .log_message(MessageType::INFO, format!("Tab size updated to {}", tab_size))
                    .await;
                self.tab_size.store(tab_size as u8, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        
        if let Some(content) = documents.get(&uri) {
            match tokenize_document(
                content, 
                self.tab_size.load(std::sync::atomic::Ordering::Relaxed),
              ) {
                Ok(tokens) => Ok(Some(SemanticTokensResult::Tokens(tokens))),
                Err(e) => {
                    self.client.log_message(MessageType::ERROR, &format!("Error tokenizing document: {}", e)).await;
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}