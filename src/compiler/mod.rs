use std::iter::Scan;

use num_enum::IntoPrimitive;

use crate::chunk::Chunk;
use crate::compiler::parse_rule::{ParseFn, ParseRule};
use crate::compiler::parser::Parser;
use crate::compiler::precedence::Precedence;
use crate::compiler::scanner::Scanner;
use crate::op_code::OpCode::{
    OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_MULTIPLY, OP_NEGATE, OP_RETURN, OP_SUBTRACT,
};
use crate::token::{Token, TokenType, TokenType::*};
use crate::value::Value;

mod parse_rule;
mod parser;
mod precedence;
pub mod scanner;

pub struct Compiler<'a> {
    parser: Parser,
    scanner: Scanner,
    compiling_chunk: &'a mut Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Self {
            parser: Parser::new(),
            scanner: Scanner::new(vec![]),
            compiling_chunk: chunk,
        }
    }
    pub fn compile(&mut self, source: Vec<u8>) -> bool {
        self.scanner.source = source;

        self.parser.had_error = false;
        self.parser.panic_mode = false;

        self.advance();
        self.expression();
        self.consume(TOKEN_EOF, "Expected end of expression.".to_string());
        self.emit_byte(OP_RETURN.into());

        if !self.parser.had_error {
            self.compiling_chunk.disassemble_chunk("code".to_string());
        }

        !self.parser.had_error
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            if let Some(token) = self.scanner.scan_token() {
                self.parser.current = Some(token.clone());

                if token.token_type != TOKEN_ERROR {
                    break;
                }

                self.error_at_current(token.message);
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, error_message: String) {
        if let Some(current_token) = &self.parser.current {
            if current_token.token_type == token_type {
                self.advance();
            }
        }

        self.error_at_current(error_message);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
        self.parse_precedence(Precedence::PREC_UNARY);
    }

    fn parse_precedence(&mut self, presedence: Precedence) {
        self.advance();
        if let Some(previous) = &self.parser.previous.clone() {
            let rule = self.get_rule(previous.clone().token_type);
            let prefix_rule = rule.prefix;

            match prefix_rule {
                ParseFn::Grouping => self.grouping(),
                ParseFn::Binary => self.binary(),
                ParseFn::Unary => self.unary(),
                ParseFn::Number => self.number(),
                ParseFn::Null => {
                    self.error("Expected expression.".to_string());
                    return;
                }
            }

            while let Some(current) = self.parser.current.clone() {
                self.advance();

                let infix_rule = self.get_rule(previous.clone().token_type).infix;
                match infix_rule {
                    ParseFn::Grouping => self.grouping(),
                    ParseFn::Binary => self.binary(),
                    ParseFn::Unary => self.unary(),
                    ParseFn::Number => self.number(),
                    ParseFn::Null => {}
                }
            }
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            TOKEN_RIGHT_PAREN,
            "Expected ')' after expression.".to_string(),
        );
    }

    fn unary(&mut self) {
        if let Some(previous) = &self.parser.previous.clone() {
            let operator_type = &previous.token_type.clone();

            self.expression();

            if *operator_type == TOKEN_MINUS {
                self.emit_byte(OP_NEGATE.into())
            }
        }
    }

    fn binary(&mut self) {
        if let Some(previous) = &self.parser.previous.clone() {
            let operator_type = &previous.token_type;
            let parse_rule = self.get_rule(operator_type.clone());
            let precedence = parse_rule.precedence;
            self.parse_precedence(precedence);

            self.expression();

            match operator_type {
                TOKEN_PLUS => self.emit_byte(OP_ADD.into()),
                TOKEN_MINUS => self.emit_byte(OP_SUBTRACT.into()),
                TOKEN_STAR => self.emit_byte(OP_MULTIPLY.into()),
                TOKEN_SLASH => self.emit_byte(OP_DIVIDE.into()),
                _ => {}
            }
        }
    }

    fn get_rule(&mut self, token_type: TokenType) -> ParseRule {
        ParseRule::from_token_type(token_type)
    }

    fn emit_byte(&mut self, byte: u8) {
        if let Some(previous) = &self.parser.previous {
            self.compiling_chunk.write(byte, previous.line);
        }
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OP_CONSTANT.into(), constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        self.compiling_chunk.add_constant(value) as u8
    }

    fn number(&mut self) {
        if let Some(value) = &self.parser.previous {
            let value = value.message.clone().parse::<Value>();
            match value {
                Ok(value) => self.emit_constant(value),
                Err(e) => panic!("constant is not valid {}", e),
            }
        }
    }

    fn error(&mut self, message: String) {
        self.error_at(&self.parser.previous.clone().unwrap(), message);
    }

    fn error_at_current(&mut self, message: String) {
        if let Some(current) = &self.parser.current.clone() {
            self.error_at(current, message)
        }
    }

    fn error_at(&mut self, token: &Token, message: String) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        eprint!("[at line {}] Error", token.line);
        if token.token_type == TOKEN_EOF {
            eprint!(" at end");
        } else if token.token_type == TOKEN_ERROR {
        } else {
            eprint!(" at '{}'", token.message);
        }

        println!(": {message}");
        self.parser.had_error = true;
    }
}
