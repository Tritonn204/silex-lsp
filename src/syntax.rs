use xelis_lexer::{Lexer, LexerError};
use xelis_ast::{Token, Literal};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokens};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, NumberOrString};

use std::collections::HashMap;

pub fn tokenize_document(content: &str, tab_size: u8) -> Result<SemanticTokens, String> {
    let mut binding = Lexer::new(content);
    let mut lexer = binding.with_tab_size(tab_size);
    
    let mut semantic_tokens = Vec::new();
    let mut line = 1;
    let mut start = 0;

    let mut context = SemanticContext::new();

    for token_result in lexer {
        let token_result = token_result.map_err(|e| format!("Lexer error: {:?}", e))?;
        let token_type = get_token_type(&token_result.token, &mut context);

        let delta_line = (token_result.line - line) as u32;
        let delta_start = if delta_line == 0 {
            (token_result.column_start - start - 1) as u32
        } else {
            token_result.column_start as u32 - 1
        };

        let st = SemanticToken {
            delta_line,
            delta_start,
            length: (token_result.column_end - token_result.column_start) as u32 + 1,
            token_type,
            token_modifiers_bitset: 0,
        };

        semantic_tokens.push(st);

        line = token_result.line;
        start = token_result.column_start - 1;
    }

    Ok(SemanticTokens {
        result_id: None,
        data: semantic_tokens,
    })
}

struct SemanticContext {
    scope_stack: Vec<HashMap<String, u32>>,
    in_function_params: bool,
    in_struct_definition: bool,
    in_enum_definition: bool,
}

impl SemanticContext {
    fn new() -> Self {
        Self {
            scope_stack: vec![HashMap::new()],
            in_function_params: false,
            in_struct_definition: false,
            in_enum_definition: false,
        }
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn add_variable(&mut self, name: String, token_type: u32) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, token_type);
        }
    }

    fn get_variable_type(&self, name: &str) -> Option<u32> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(&token_type) = scope.get(name) {
                return Some(token_type);
            }
        }
        None
    }
}

fn get_token_type(token: &Token, context: &mut SemanticContext) -> u32 {
    match token {
        Token::Let | Token::Const | Token::Function | Token::Entry | 
        Token::Struct | Token::Enum | Token::For | Token::ForEach | 
        Token::While | Token::If | Token::Else | Token::Return | 
        Token::Continue | Token::Break => 0, // Keyword
        Token::Identifier(name) => {
            if context.in_function_params {
                context.add_variable(name.to_string(), 6); // Parameter
                7
            } else if context.in_struct_definition || context.in_enum_definition {
                8 // Type
            } else {
                context.get_variable_type(name).unwrap_or(2) // Variable or Function (assuming 2 is for variables)
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
        _ => 9, // Other (you might want to adjust this)
    }
}