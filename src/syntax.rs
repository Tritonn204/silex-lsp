use tower_lsp::lsp_types::{SemanticToken, SemanticTokens};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, NumberOrString};

use super::*;

use xelis_parser::mapper::GlobalMapper;
use xelis_types::Type;

use std::collections::{HashMap, HashSet};

pub fn tokenize_document(content: &str, funcs: &HashMap<Option<Type>, HashMap<String, HashSet<usize>>>) -> Result<(SemanticTokens, Vec<Diagnostic>), String> {
  let lexer = Lexer::new(content);

  let mut local_funcs = (*funcs).clone();

  let mut semantic_tokens = Vec::new();
  let mut diagnostics = Vec::new();

  let mut context = SemanticContext::new(&mut local_funcs);

  for token_result in lexer {
    let token_result = token_result.map_err(|e| format!("Lexer error: {:?}", e))?;

    handle_token(&token_result.token, &mut context);
    let token_type = get_token_type(&token_result.token, &mut context);

    let token_start = token_result.column_start;
    let token_end = token_result.column_end;

    let delta_line = (token_result.line - context.line) as u32;
    let delta_start = if delta_line == 0 {
      (token_start - context.start) as u32
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

    context.start = token_start;
    context.line = token_result.line;

    if !token_result.token.is_operator() {
      context.prev_type = token_type;
    }
    if context.prev_token == token_result.token {
      context.token_chain = Some(context.prev_token.to_owned());
    } else {
      context.token_chain = None;
    }

    context.prev_token = token_result.token.to_owned();
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
  pub line: usize,
  pub start: usize,

  namespace: Vec<&'a str>,
  prev_token: Token<'a>,
  prev_type: u32,

  token_chain: Option<Token<'a>>,
  
  funcs: &'a mut HashMap<Option<Type>, HashMap<String, HashSet<usize>>>,
  scope_stack: Vec<HashMap<&'a str, u32>>,
  call_stack: Vec<(&'a str, usize)>,

  current_func_name: String,
  current_func_type: u32,

  current_param_count: usize,
  valid_param_counts: HashSet<usize>,
  count_params: bool,

  in_function_declaration: bool,
  in_function_params: bool,
  in_struct_constructor: bool,
  in_enum_constructor: bool,
  
  open_p_count: usize,
  close_p_count: usize,
}

impl<'a> SemanticContext<'a> {
  fn new(funcs: &'a mut HashMap<Option<Type>, HashMap<String, HashSet<usize>>>) -> Self {
    Self {
      line: 1,
      start: 1,

      funcs: funcs,
      prev_token: Token::Value(Literal::Null),
      prev_type: 255,

      namespace: Vec::new(),
      scope_stack: vec![HashMap::new()],
      call_stack: Vec::new(),

      current_func_name: String::new(),
      current_func_type: 0,

      current_param_count: 0,
      valid_param_counts: HashSet::new(),
      count_params: true,

      in_function_declaration: false,
      in_function_params: false,
      in_struct_constructor: false,
      in_enum_constructor: false,
      open_p_count: 0,
      close_p_count: 0,
    }
  }

  fn to_complete_name(&self, name: &'a str) -> String {
    if self.namespace.is_empty() {
      name.to_string()
    } else {
      self.namespace.join("::") + "::" + name
    }
  }

  fn enter_scope(&mut self) {
    self.scope_stack.push(HashMap::new());
  }

  fn enter_call_scope(&mut self, name: &'a str) {
    self.call_stack.push((name, 0));
  }

  fn exit_scope(&mut self) {
    self.scope_stack.pop();
  }

  fn exit_call_scope(&mut self) {
    self.call_stack.pop();
  }

  fn register_function(&mut self, name: &'a str, namespace: Vec<&'a str>, param_count: usize) {
    let full_name = self.to_complete_name(name);

    self.funcs.entry(None).or_insert(HashMap::new()) // only non type-gated functions can be defined in Silex code
      .entry(full_name).or_insert(HashSet::new())
        .insert(param_count);
  }

  fn add_variable(&mut self, name: &'a str, token_type: u32) {
    if let Some(scope) = self.scope_stack.last_mut() {
      scope.entry(name).or_insert(token_type);
    }
  }

  fn get_function_params(&self, on_type: Option<Type>, name: &'a str, namespace: Vec<&'a str>) -> Option<HashSet<usize>> {
    if let Some(glossary) = self.funcs.get(&on_type) {
      let full_name = if namespace.is_empty() {
        name.to_string()
      } else {
        namespace.join("::") + "::" + name
      };

      return glossary.get(&full_name).cloned();
    }
    None
  } 

  fn get_variable_type(&self, name: &'a str) -> Option<u32> {
    for scope in self.scope_stack.iter().rev() {
      if let Some(&token_type) = scope.get(&name) {
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
          context.in_function_declaration = true;
          context.current_func_name = if context.namespace.is_empty() {
            name.to_string()
          } else {
            context.namespace.join("::") + "::" + name
          };
        },
        Token::Struct | Token::Enum => {
          context.add_variable(name, 8);
        },
        Token::EnterNamespace => {
          context.add_variable(name, 9);
          context.namespace.push(name); // TODO: handle the exit and popping namespace entries
        },
        Token::Let | Token::Const => {
          context.add_variable(name, 2);
        },
        
        Token::For |
        Token::ForEach | Token::While => {
          context.add_variable(name, 10);
        },

        Token::ParenthesisOpen | Token::Comma => {
          if context.in_function_params {
            if context.in_function_declaration {
              context.current_param_count += 1;
            } else {
              if context.count_params {
                match context.call_stack.last_mut() {
                  Some(mut func_call) => {
                    func_call.1 += 1;
                  },
                  None => {}
                }
              }
            }
            context.add_variable(name, 10);
          }
        }
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
              context.current_param_count = 0;
              context.current_func_name = name.to_string();
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
        if context.in_function_declaration {

          context.in_function_declaration = false;
        }
        context.current_func_name = String::new();
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

    _ => {}
  }
}

fn get_token_type<'a>(token: &Token<'a>, context: &mut SemanticContext<'a>) -> u32 {
  if token.is_operator() {
    return 6; // Operator
  }

  match token {
    Token::Let | Token::Const | Token::Function | Token::Entry | 
    Token::Struct | Token::Enum | Token::For | Token::ForEach | 
    Token::While | Token::If | Token::Else | Token::Return | 
    Token::Continue | Token::Break => 0, // Keyword,

    Token::Let | Token::As | Token::In => 12, // Variable Declaration Accessory

    Token::Identifier(name) => {
      if *name == "import" {
        0 // Keyword
      } else {
        match context.get_function_params(
          None,
          name,
          Vec::new(),
        ) {
          Some(expected) => {
            context.valid_param_counts = expected;
            context.enter_call_scope(name);
            return 1;
          }, // Function
          None => {},
        }

        match context.get_variable_type(name) {
          Some(t) => t,
          None => {
            context.add_variable(name, 11); // UnknownId
            11
          }
        }
      }
    },

    Token::Value(val) => {
      match val {
        Literal::String(_) => 3, // String

        Literal::Number(_) |
        Literal::U8(_) |
        Literal::U16(_) |
        Literal::U32(_) |
        Literal::U64(_) |
        Literal::U128(_) |
        Literal::U256(_) => 5, // Number

        Literal::Bool(_) |
        Literal::Null => 0, // Keyword
      }
    },

    _ => {
      if token.is_type() {
        return 8; // Type
      }
      255
    },
  }
}