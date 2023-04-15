use crate::token::{TokenType, TokenType::*};

use crate::compiler::precedence::Precedence;
use std::collections::HashMap;
use std::iter::Map;

#[derive(Debug, PartialEq)]
pub enum ParseFn {
    Grouping,
    Null,
    Binary,
    Unary,
    Number,
}

pub struct ParseRule {
    pub(crate) prefix: ParseFn,
    pub(crate) infix: ParseFn,
    pub(crate) precedence: Precedence,
}
impl ParseRule {
    pub fn from_token_type(token_type: TokenType) -> Self {
        match token_type {
            TOKEN_LEFT_PAREN => ParseRule {
                prefix: ParseFn::Grouping,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_RIGHT_PAREN => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_LEFT_BRACE => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_RIGHT_BRACE => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_COMMA => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_DOT => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_MINUS => ParseRule {
                prefix: ParseFn::Unary,
                infix: ParseFn::Binary,
                precedence: Precedence::PREC_TERM,
            },
            TOKEN_PLUS => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Binary,
                precedence: Precedence::PREC_TERM,
            },
            TOKEN_SEMICOLON => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_SLASH => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Binary,
                precedence: Precedence::PREC_FACTOR,
            },
            TOKEN_STAR => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Binary,
                precedence: Precedence::PREC_FACTOR,
            },
            TOKEN_BANG => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_BANG_EQUAL => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_EQUAL => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_EQUAL_EQUAL => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_GREATER => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_GREATER_EQUAL => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_LESS => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_LESS_EQUAL => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_IDENTIFIER => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_STRING => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_NUMBER => ParseRule {
                prefix: ParseFn::Number,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_AND => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_CLASS => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_ELSE => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_FALSE => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_FOR => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_FUN => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_IF => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_NIL => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_OR => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_PRINT => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_RETURN => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_SUPER => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_THIS => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_TRUE => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_VAR => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_WHILE => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_ERROR => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
            TOKEN_EOF => ParseRule {
                prefix: ParseFn::Null,
                infix: ParseFn::Null,
                precedence: Precedence::PREC_NONE,
            },
        }
    }
}
