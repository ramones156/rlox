use std::iter::Scan;

use num_enum::IntoPrimitive;

use crate::chunk::Chunk;
use crate::compiler::parse_rule::{ParseFn, ParseRule};
use crate::compiler::parser::Parser;
use crate::compiler::precedence::Precedence;
use crate::compiler::precedence::Precedence::PREC_NONE;
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
            self.parser.current = self.scanner.scan_token();

            if let Some(current) = self.parser.current.clone() {
                if current.token_type != TOKEN_ERROR {
                    break;
                }

                self.error_at_current(current.message);
            } else {
                break;
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, error_message: String) {
        if let Some(current_token) = &self.parser.current {
            if current_token.token_type == token_type {
                self.advance();
                return;
            }
        }

        self.error_at_current(error_message);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        if let Some(previous) = &self.parser.previous.clone() {
            let rule = self.get_rule(&previous.clone().token_type);
            let prefix_rule = rule.prefix;

            match prefix_rule {
                ParseFn::Grouping => self.grouping(),
                ParseFn::Unary => self.unary(),
                ParseFn::Number => self.number(),
                ParseFn::Null => {
                    self.error("Expected expression.".to_string());
                    return;
                }
                _ => unreachable!(),
            }
        }

        while let Some(current) = self.parser.current.clone() {
            if precedence > self.get_rule(&current.token_type).precedence {
                break;
            }
            self.advance();
            if let Some(previous) = &self.parser.previous.clone() {
                let infix_rule = self.get_rule(&previous.clone().token_type).infix;
                match infix_rule {
                    ParseFn::Binary => self.binary(),
                    ParseFn::Null => {}
                    _ => unreachable!(),
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

            self.parse_precedence(Precedence::PREC_UNARY);

            match *operator_type {
                TOKEN_MINUS => self.emit_byte(OP_NEGATE.into()),
                _ => unreachable!(),
            }
        }
    }

    fn binary(&mut self) {
        if let Some(previous) = &self.parser.previous.clone() {
            let operator_type = &previous.token_type;
            let parse_rule = self.get_rule(&operator_type.clone());
            let precedence: u8 = parse_rule.precedence.into();

            self.parse_precedence(Precedence::try_from(precedence + 1).unwrap());

            match operator_type {
                TOKEN_PLUS => self.emit_byte(OP_ADD.into()),
                TOKEN_MINUS => self.emit_byte(OP_SUBTRACT.into()),
                TOKEN_STAR => self.emit_byte(OP_MULTIPLY.into()),
                TOKEN_SLASH => self.emit_byte(OP_DIVIDE.into()),
                _ => unreachable!(),
            }
        }
    }

    fn get_rule(&mut self, token_type: &TokenType) -> ParseRule {
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
        if let Some(token) = &self.parser.previous {
            match token.message.clone().parse::<Value>() {
                Ok(value) => self.emit_constant(value),
                Err(e) => panic!("constant {} is not valid {}", &token.message, e),
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
        eprint!("[{}:{}] Error", token.line, token.start);
        if token.token_type == TOKEN_EOF {
            eprint!(" at end");
        } else if token.token_type == TOKEN_ERROR {
        } else {
            eprint!(" at '{:?}'", token.token_type);
        }

        println!(": {message}");
        self.parser.had_error = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::Chunk;
    use crate::compiler::Compiler;
    use crate::op_code::OpCode::{OP_ADD, OP_MULTIPLY, OP_NEGATE, OP_RETURN};

    #[test]
    fn parse_precedence_should_succeed() {
        let code = "-54.55 * (2.0 + 6)"; // -a.b * (c + d)
        let mut chunk = Chunk::default();
        let mut compiler = Compiler::new(&mut chunk);

        let result = compiler.compile(code.to_string().into_bytes());
        assert!(result);

        // chunk constants
        assert_eq!(chunk.constants.values[0], 54.55);
        assert_eq!(chunk.constants.values[1], 2.0);
        assert_eq!(chunk.constants.values[2], 6.0);

        // chunk code instructions
        assert_eq!(chunk.code[0..2], [0, 0]);
        assert_eq!(chunk.code[2], OP_NEGATE.into());
        assert_eq!(chunk.code[3..5], [0, 1]);
        assert_eq!(chunk.code[5..7], [0, 2]);
        assert_eq!(chunk.code[7], OP_ADD.into());
        assert_eq!(chunk.code[8], OP_MULTIPLY.into());
        assert_eq!(chunk.code[9], OP_RETURN.into());
    }
}
