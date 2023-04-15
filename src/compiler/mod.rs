mod parse_rule;
mod precedence;
pub mod scanner;

use crate::chunk::Chunk;
use crate::compiler::parse_rule::{ParseFn, ParseRule};
use crate::compiler::precedence::Precedence;
use crate::compiler::scanner::Scanner;
use crate::op_code::OpCode::{
    OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_MULTIPLY, OP_NEGATE, OP_RETURN, OP_SUBTRACT,
};
use crate::token::{Token, TokenType, TokenType::*};
use crate::value::Value;
use num_enum::IntoPrimitive;
use std::iter::Scan;

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
        self.consume(TOKEN_EOF, "Expect end of expression.".to_string());
        self.emit_byte(OP_RETURN.into());

        if !self.parser.had_error {
            self.compiling_chunk.disassemble_chunk("code".to_string());
        }

        !self.parser.had_error
    }

    fn advance(&mut self) {
        loop {
            if let Some(token) = self.scanner.scan_token() {
                if token.token_type != TOKEN_ERROR {
                    break;
                }

                self.error_at_current(token.message);
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: String) {
        if let Some(current_token) = &self.parser.current {
            if current_token.token_type == token_type {
                self.advance();
            }
        }

        self.error_at_current(message);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
        self.parse_precedence(Precedence::PREC_UNARY);
    }

    fn parse_precedence(&mut self, presedence: Precedence) {
        self.advance();
        if let Some(previous) = &self.parser.previous.clone() {
            let prefix_rule = self.get_rule(previous.clone().token_type).prefix;
            if prefix_rule == ParseFn::Null {
                eprintln!("Expected expression");
                return;
            }

            // self.prefix_rule();

            loop {
                let precedence = self.get_rule(previous.clone().token_type).precedence;
                if precedence == Precedence::PREC_NONE {
                    break;
                }
                self.advance();
                let infix_rule = self.get_rule(previous.clone().token_type).infix;
                // self.infix_rule();
            }
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            TOKEN_RIGHT_PAREN,
            "Expect ')' after expression.".to_string(),
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
        eprint!("[line at {}] Error", token.line);
        if token.token_type == TOKEN_EOF {
            eprint!(" at end");
        } else if token.token_type == TOKEN_ERROR {
        } else {
            eprint!(" at {}", token.message);
        }
    }
}

struct Parser {
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    fn new() -> Self {
        Self {
            current: None,
            previous: None,
            had_error: false,
            panic_mode: true,
        }
    }
}
