use tower_lsp::{LspService, Server};
use tokio::sync::Mutex;
use std::collections::HashMap;

use env_logger;

use silex_lsp::lsp::SilexLanguageServer;

#[tokio::main]
async fn main() {
  env_logger::Builder::new().target(env_logger::Target::Stderr).init();
  std::panic::set_hook(Box::new(|info| {
    eprintln!("Panic occurred: {:?}", info);
  }));

  let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

  let (service, socket) = LspService::new(|client| SilexLanguageServer { 
    client, 
    documents: Mutex::new(HashMap::new()), 
    tab_size: 4.into(),
    funcs: HashMap::new().into()
  });
  Server::new(stdin, stdout, socket).serve(service).await;
}
