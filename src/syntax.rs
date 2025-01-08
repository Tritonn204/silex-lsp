use tower_lsp::lsp_types::{SemanticToken, SemanticTokens};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, NumberOrString};

use super::*;

use xelis_parser::mapper::GlobalMapper;
use std::collections::HashMap;

pub fn tokenize_document(content: &str) -> Result<(SemanticTokens, Vec<Diagnostic>), String> {
  let lexer = Lexer::new(content);

  let mut semantic_tokens = Vec::new();
  let mut diagnostics = Vec::new();

  let mut line = 1;
  let mut start = 1;

  let mut context = SemanticContext::new();

  for token_result in lexer {
    let token_result = token_result.map_err(|e| format!("Lexer error: {:?}", e))?;

    handle_token(&token_result.token, &mut context);
    let token_type = get_token_type(&token_result.token, &mut context);

    let token_start = token_result.column_start;
    let token_end = token_result.column_end;

    let delta_line = (token_result.line - line) as u32;
    let delta_start = if delta_line == 0 {
      (token_start - start) as u32
    } else {
      token_start as u32 - 1
    };

    let length = (token_end - token_start + 1) as u32;

    semantic_tokens.push(SemanticToken {
      delta_line,
      delta_start,
      length,
      token_type,
      token_modifiers_bitset: 0,
    });

    if token_type == 11 {
      diagnostics.push(Diagnostic {
        range: Range {
          start: Position {
            line: (token_result.line - 1) as u32,
            character: (token_start - 1) as u32,
          },
          end: Position {
            line: (token_result.line - 1) as u32,
            character: token_end as u32,
          },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("unknown-identifier".to_string())),
        code_description: None,
        source: Some("silex".to_string()),
        message: format!("Unknown identifier: '{}'", token_result.token),
        related_information: None,
        tags: None,
        data: None,
      });
    }

    start = token_start;
    line = token_result.line;

    if !token_result.token.is_operator() {
      context.prev_token = token_result.token.to_owned();
    }
  }

  Ok((
    SemanticTokens {
      result_id: None,
      data: semantic_tokens,
    },
    diagnostics,
  ))
}

struct SemanticContext<'a> {
  namespace: Vec<&'a str>,
  prev_token: Token<'a>,
  scope_stack: Vec<HashMap<&'a str, u32>>,

  in_function_params: bool,
  in_struct_constructor: bool,
  in_enum_constructor: bool,
  open_p_count: usize,
  close_p_count: usize,
}

impl<'a> SemanticContext<'a> {
  fn new() -> Self {
    Self {
      namespace: Vec::new(),
      prev_token: Token::Value(Literal::Null),
      scope_stack: vec![HashMap::new()],
      in_function_params: false,
      in_struct_constructor: false,
      in_enum_constructor: false,
      open_p_count: 0,
      close_p_count: 0,
    }
  }

  fn enter_scope(&mut self) {
    self.scope_stack.push(HashMap::new());
  }

  fn exit_scope(&mut self) {
    self.scope_stack.pop();
  }

  fn add_variable(&mut self, name: &'a str, token_type: u32) {
    if let Some(scope) = self.scope_stack.last_mut() {
      scope.entry(name).or_insert(token_type);
    }
  }

  fn get_variable_type(&self, name: &'a str) -> Option<u32> {
    for scope in self.scope_stack.iter().rev() {
      if let Some(&token_type) = scope.get(name) {
        return Some(token_type);
      }
    }
    None
  }
}

fn handle_token<'a>(token: &Token<'a>, context: &mut SemanticContext<'a>) {
  
  match token {
    Token::Identifier(name) => {
      match context.prev_token {
        Token::Entry | Token::Function => {
          context.add_variable(name, 1);
        },
        Token::Struct | Token::Enum => {
          context.add_variable(name, 8);
        },
        Token::EnterNamespace => {
          context.add_variable(name, 9);
          context.namespace.push(name); // TODO: handle the exit and popping namespace entries
        },
        Token::Let | Token::Const | Token::For |
        Token::ForEach | Token::While => {
          context.add_variable(name, 2);
        },
        _ => {}
      }
    },

    Token::ParenthesisOpen => {
      context.open_p_count += 1;
      match context.prev_token {
        Token::Identifier(name) => {
          match context.get_variable_type(name) {
            Some(1) => {
              context.in_function_params = true;
              context.open_p_count = 1;
              context.close_p_count = 0;
            },
            _ => {}
          }
        },
        _ => {}
      }
    },

    Token::ParenthesisClose => {
      context.close_p_count += 1;
      if context.in_function_params && context.open_p_count == context.close_p_count {
        context.in_function_params = false;
        context.open_p_count = 0;
        context.close_p_count = 0;
      }
    }

    Token::BraceOpen => {
      context.enter_scope();
      match context.prev_token {
        Token::Identifier(name) => {
          match context.get_variable_type(name) {
            Some(9) => {
              context.namespace.push(name);
            },

            _ => {}
          }
        },

        _ => {}
      }
    },

    Token::BraceClose => {
      context.exit_scope();
    },

    // Token::Value(value) => {
    //   match value {
    //     Literal::String() => {},
    //     _ => {}
    //   }
    // },

    _ => {}
  }
}

fn get_token_type<'a>(token: &Token<'a>, context: &mut SemanticContext<'a>) -> u32 {
  match token {
    Token::Let | Token::Const | Token::Function | Token::Entry | 
    Token::Struct | Token::Enum | Token::For | Token::ForEach | 
    Token::While | Token::If | Token::Else | Token::Return | 
    Token::Continue | Token::Break => 0, // Keyword

    Token::Bool | Token::String | Token::Optional |
    Token::Map | Token::Blob => 8, // Type

    Token::Identifier(name) => {
      if *name == "import" {
        0
      } else if context.in_function_params {
        context.add_variable(name, 10); // Parameter
        10
      } else {
        match context.get_variable_type(name) {
          Some(t) => t,
          None => {
            context.add_variable(name, 11); // UnknownId
            11
          }
        }
      }
    },

    Token::Number(_) => 5, // Number

    Token::Value(_) => {
      match token {
        Token::Value(Literal::String(_)) => 3, // String
        _ => 4, // Literal
      }
    }, // String or other literals

    Token::OperatorAssign | Token::OperatorPlus | Token::OperatorMinus |
    Token::OperatorMultiply | Token::OperatorDivide | Token::OperatorModulo |
    Token::OperatorPow | Token::OperatorBitwiseXor | Token::OperatorBitwiseOr |
    Token::OperatorBitwiseAnd | Token::OperatorBitwiseShl | Token::OperatorBitwiseShr |
    Token::OperatorEquals | Token::OperatorNotEquals | Token::OperatorGreaterThan |
    Token::OperatorLessThan | Token::OperatorGreaterOrEqual | Token::OperatorLessOrEqual |
    Token::OperatorAnd | Token::OperatorOr => 6, // Operator
    _ => 255,
  }
}