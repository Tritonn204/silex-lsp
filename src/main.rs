use tower_lsp::{LspService, Server};
use tokio::sync::Mutex;
use std::collections::HashMap;

mod lsp;

use lsp::SilexLanguageServer;

#[tokio::main]
async fn main() {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| SilexLanguageServer { 
      client, 
      documents: Mutex::new(HashMap::new()), 
      tab_size: 4.into(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
