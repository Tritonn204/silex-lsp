use tower_lsp::{LspService, Server};
use tokio::sync::Mutex;
use std::collections::HashMap;

mod lsp;

use lsp::SilexLanguageServer;

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(|client| SilexLanguageServer { 
      client, 
      documents: Mutex::new(HashMap::new()), 
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
