use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;

use super::*;
use crate::syntax::{tokenize_document};

use xelis_ast::Signature;
use xelis_types::{Constant, EnumType, OpaqueType, Opaque, StructType, Type};

// Owned mirror of the Silex Function type for xstd registration
#[derive(Debug)]
pub struct FunctionData {
  pub parameters: Vec<(String, Type)>,
  pub return_type: Option<Type>
}

#[derive(Debug)]
pub struct SilexLanguageServer {
  pub client: Client,
  pub documents: Mutex<HashMap<Url, String>>,
  pub tab_size: AtomicU8,

  //
  pub funcs: Mutex<HashMap<Option<Type>, HashMap<(String, Vec<String>), FunctionData>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for SilexLanguageServer {
  async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {

    // register xstd functions in the server
    let mut funcs = self.funcs.lock().await;
    let env = EnvironmentBuilder::default();
    for fdata in env.get_functions_mapper().get_declared_functions() {
      let ty_funcs = funcs.entry(fdata.0.cloned()).or_insert(HashMap::new());
      for function in fdata.1 {
        let key = (function.name.to_string(), function.namespace.iter().map(|s| s.to_string()).collect());

        ty_funcs
          .insert(key, FunctionData {
            parameters: function.parameters.iter()
              .map(|(name, ty)| (name.to_string(), ty.clone()))
              .collect(),
            return_type: function.return_type.clone(),
          });
      }
    }

    self.client
      .log_message(MessageType::INFO, format!("XSTD funcs: {:?}", funcs))
      .await;

    // transmit lsp capabilities
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
                  SemanticTokenType::KEYWORD,               // 0
                  SemanticTokenType::FUNCTION,              // 1
                  SemanticTokenType::VARIABLE,              // 2
                  SemanticTokenType::STRING,                // 3
                  SemanticTokenType::new("literal"),        // 4
                  SemanticTokenType::NUMBER,                // 5
                  SemanticTokenType::OPERATOR,              // 6
                  SemanticTokenType::COMMENT,               // 7
                  SemanticTokenType::TYPE,                  // 8
                  SemanticTokenType::NAMESPACE,             // 9
                  SemanticTokenType::PARAMETER,             // 10
                  SemanticTokenType::new("unknownId"),      // 11
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
  
  async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
    let uri = params.text_document.uri.clone();
    let documents = self.documents.lock().await;

    if let Some(content) = documents.get(&uri) {
      match tokenize_document(content) {
        Ok((tokens, diagnostics)) => {
          self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;

          Ok(Some(SemanticTokensResult::Tokens(tokens)))
        }
        Err(e) => {
          self.client
            .log_message(MessageType::ERROR, format!("Error processing document: {}", e))
            .await;
          Ok(None)
        }
      }
    } else {
      Ok(None)
    }
  }
}