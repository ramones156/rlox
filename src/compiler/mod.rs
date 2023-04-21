use std::iter::Scan;

use num_enum::IntoPrimitive;

use crate::chunk::Chunk;
use crate::compiler::parse_rule::{ParseFn, ParseRule};
use crate::compiler::parser::Parser;
use crate::compiler::precedence::Precedence;
use crate::compiler::precedence::Precedence::PREC_NONE;
use crate::compiler::scanner::Scanner;
use crate::object::ObjectType::OBJ_STRING;
use crate::object::{Object, ObjectType};
use crate::op_code::OpCode::{
    OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_EQUAL, OP_FALSE, OP_GREATER, OP_LESS, OP_MULTIPLY,
    OP_NEGATE, OP_NIL, OP_NOT, OP_RETURN, OP_SUBTRACT, OP_TRUE,
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
                ParseFn::Literal => self.literal(),
                ParseFn::String => self.string(),
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
                TOKEN_BANG => self.emit_byte(OP_NOT.into()),
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
                TOKEN_BANG_EQUAL => self.emit_bytes(OP_EQUAL.into(), OP_NOT.into()),
                TOKEN_EQUAL_EQUAL => self.emit_byte(OP_EQUAL.into()),
                TOKEN_GREATER => self.emit_byte(OP_GREATER.into()),
                TOKEN_GREATER_EQUAL => self.emit_bytes(OP_LESS.into(), OP_NOT.into()),
                TOKEN_LESS => self.emit_byte(OP_LESS.into()),
                TOKEN_LESS_EQUAL => self.emit_bytes(OP_GREATER.into(), OP_NOT.into()),
                _ => unreachable!(),
            }
        }
    }

    fn literal(&mut self) {
        if let Some(previous) = self.parser.previous.clone() {
            match previous.token_type {
                TOKEN_FALSE => self.emit_byte(OP_FALSE.into()),
                TOKEN_TRUE => self.emit_byte(OP_TRUE.into()),
                TOKEN_NIL => self.emit_byte(OP_NIL.into()),
                _ => {}
            }
        }
    }

    fn string(&mut self) {
        if let Some(previous) = self.parser.previous.clone() {
            self.emit_constant(Value::VAL_OBJECT(self.clone_string(previous.message)))
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

    fn clone_string(&self, string: String) -> Object {
        Object {
            object_type: OBJ_STRING(string),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::Chunk;
    use crate::compiler::Compiler;
    use crate::object::{Object, ObjectType};
    use crate::op_code::OpCode;
    use crate::op_code::OpCode::{
        OP_ADD, OP_EQUAL, OP_GREATER, OP_MULTIPLY, OP_NEGATE, OP_NIL, OP_NOT, OP_RETURN,
        OP_SUBTRACT,
    };
    use crate::value::Value;

    #[test]
    fn parse_precedence_number_order_should_succeed() {
        let code = "-54.55 * (2.0 + 6)"; // -a.b * (c + d)
        let mut chunk = Chunk::default();
        let mut compiler = Compiler::new(&mut chunk);

        let result = compiler.compile(code.to_string().into_bytes());
        assert!(result);

        // chunk constants
        assert_eq!(chunk.constants.values[0], Value::VAL_NUMBER(54.55));
        assert_eq!(chunk.constants.values[1], Value::VAL_NUMBER(2.0));
        assert_eq!(chunk.constants.values[2], Value::VAL_NUMBER(6.0));

        // chunk code instructions
        assert_eq!(chunk.code[0..2], [0, 0]);
        assert_eq!(chunk.code[2], OP_NEGATE.into());
        assert_eq!(chunk.code[3..5], [0, 1]);
        assert_eq!(chunk.code[5..7], [0, 2]);
        assert_eq!(chunk.code[7], OP_ADD.into());
        assert_eq!(chunk.code[8], OP_MULTIPLY.into());
        assert_eq!(chunk.code[9], OP_RETURN.into());
    }

    #[test]
    fn parse_precedence_boolean_should_succeed() {
        let code = "!(5 - 4 > 3 * 2 == !nil)";
        let mut chunk = Chunk::default();
        let mut compiler = Compiler::new(&mut chunk);

        let result = compiler.compile(code.to_string().into_bytes());
        assert!(result);

        // chunk constants
        assert_eq!(chunk.constants.values[0], Value::VAL_NUMBER(5.0));
        assert_eq!(chunk.constants.values[1], Value::VAL_NUMBER(4.0));
        assert_eq!(chunk.constants.values[2], Value::VAL_NUMBER(3.0));
        assert_eq!(chunk.constants.values[3], Value::VAL_NUMBER(2.0));

        assert_eq!(chunk.code[0..2], [0, 0]);
        assert_eq!(chunk.code[2..4], [0, 1]);
        assert_eq!(chunk.code[4], OP_SUBTRACT.into());
        assert_eq!(chunk.code[5..7], [0, 2]);
        assert_eq!(chunk.code[7..9], [0, 3]);
        assert_eq!(chunk.code[9], OP_MULTIPLY.into());
        assert_eq!(chunk.code[10], OP_GREATER.into());
        assert_eq!(chunk.code[11], OP_NIL.into());
        assert_eq!(chunk.code[12], OP_NOT.into());
        assert_eq!(chunk.code[13], OP_EQUAL.into());
        assert_eq!(chunk.code[14], OP_NOT.into());
        assert_eq!(chunk.code[15], OP_RETURN.into());
    }

    #[test]
    fn parse_precedence_string_assert_should_succeed() {
        let code = r#""test" == "test""#;
        let mut chunk = Chunk::default();
        let mut compiler = Compiler::new(&mut chunk);

        let result = compiler.compile(code.to_string().into_bytes());
        assert!(result);

        // chunk constants
        let string = Value::VAL_OBJECT(Object {
            object_type: ObjectType::OBJ_STRING(String::from(r#""test""#)),
        });
        assert_eq!(chunk.constants.values[0], string);
        assert_eq!(chunk.constants.values[1], string);

        assert_eq!(chunk.code[0..2], [0, 0]);
        assert_eq!(chunk.code[2..4], [0, 1]);
        assert_eq!(chunk.code[4], OP_EQUAL.into());
        assert_eq!(chunk.code[5], OP_RETURN.into());
    }

    #[test]
    fn parse_precedence_string_concatenation_should_succeed() {
        let code = r#""st" + "ri"+"ng""#;
        let mut chunk = Chunk::default();
        let mut compiler = Compiler::new(&mut chunk);

        let result = compiler.compile(code.to_string().into_bytes());
        assert!(result);

        // chunk constants
        assert_eq!(
            chunk.constants.values[0],
            Value::VAL_OBJECT(Object {
                object_type: ObjectType::OBJ_STRING(String::from(r#""st""#)),
            })
        );
        assert_eq!(
            chunk.constants.values[1],
            Value::VAL_OBJECT(Object {
                object_type: ObjectType::OBJ_STRING(String::from(r#""ri""#)),
            })
        );
        assert_eq!(
            chunk.constants.values[2],
            Value::VAL_OBJECT(Object {
                object_type: ObjectType::OBJ_STRING(String::from(r#""ng""#)),
            })
        );

        assert_eq!(chunk.code[0..2], [0, 0]);
        assert_eq!(chunk.code[2..4], [0, 1]);
        assert_eq!(chunk.code[4], OP_ADD.into());
        assert_eq!(chunk.code[5..7], [0, 2]);
        assert_eq!(chunk.code[7], OP_ADD.into());
        assert_eq!(chunk.code[8], OP_RETURN.into());
    }
}
