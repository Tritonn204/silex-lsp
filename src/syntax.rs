use xelis_lexer::{Lexer, LexerError};
use xelis_ast::Token;
use tower_lsp::lsp_types::{SemanticToken, SemanticTokens};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, NumberOrString};

pub fn tokenize_document(document: &str) -> Result<SemanticTokens, String>  {
  let lexer = Lexer::new(document);

  let tokens = lexer
      .filter_map(|result| match result {
          Ok(token_result) => Some(SemanticToken {
              delta_line: token_result.line as u32,
              delta_start: token_result.column_start as u32,
              length: (token_result.column_end - token_result.column_start) as u32,
              token_type: match token_result.token {
                  Token::Let => 0, // Match this with your LEGEND_TYPE index for `let`
                  Token::Function => 1, // Match this with your LEGEND_TYPE index for `fn`
                  Token::If | Token::Else => 2, // Match this with your LEGEND_TYPE index for control flow keywords
                  Token::Value(_) => 3, // Match this with your LEGEND_TYPE index for literals
                  Token::Identifier(_) => 4, // Match this with your LEGEND_TYPE index for variables
                  Token::OperatorAssign | Token::OperatorPlus | Token::OperatorMinus => 5, // Match with operators
                  _ => return None, // Skip tokens that aren't mapped
              },
              token_modifiers_bitset: 0, // No modifiers for now
          }),
          Err(_) => None,
      })
      .collect::<Vec<SemanticToken>>();

  Ok(SemanticTokens { result_id: None, data: tokens })
}

pub fn generate_diagnostics(document: &str) -> Vec<Diagnostic> {
    let lexer = Lexer::new(document);
    let mut diagnostics = vec![];

    for token_result in lexer {
        if let Err(err) = token_result {
            let diagnostic = Diagnostic {
                range: Range {
                    start: Position {
                        line: err.line as u32 - 1,
                        character: err.column as u32 - 1,
                    },
                    end: Position {
                        line: err.line as u32 - 1,
                        character: err.column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("syntax_error".to_string())),
                source: Some("silex-lsp".into()),
                message: format!("Lexer error: {}", err.kind),
                ..Default::default()
            };

            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}
