use std::cell::{Ref, RefCell};
use std::fmt::Error;
use std::ptr::eq;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use num_enum::TryFromPrimitiveError;
use thiserror::Error;

use crate::chunk::{Chunk, Instruction};
use crate::compiler::Compiler;
use crate::object::{Object, ObjectType};
use crate::op::BinaryOp;
use crate::op_code::OpCode;
use crate::value::Value;
use crate::value::Value::{VAL_BOOL, VAL_OBJECT};
use crate::vm::InterpretError::COMPILE_ERROR;

const MAX_STACK_SIZE: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: [Option<Value>; MAX_STACK_SIZE],
    sp: usize,
}

impl VM {
    pub fn interpret(source: Vec<u8>) -> Result<()> {
        let mut chunk = Chunk::default();

        let mut compiler = Compiler::new(&mut chunk);

        if !compiler.compile(source) {
            return Err(COMPILE_ERROR.into());
        }

        let stack = Self::init_stack();
        let mut vm = Self {
            chunk,
            ip: 0,
            stack,
            sp: 0,
        };

        vm.run()?;
        Ok(())
    }

    fn push(&mut self, value: Value) {
        self.stack[self.sp] = Some(value);
        self.sp += 1;
    }

    fn pop(&mut self) -> &Value {
        self.sp -= 1;
        self.stack[self.sp].as_ref().unwrap()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            print!("        ");
            for i in 0..self.sp {
                print!("[ ");
                print!("{:?}", self.stack[i].clone().unwrap());
                print!(" ]");
            }
            println!();

            self.chunk.disassemble_instruction(self.ip);
            let instruction = self.read_instruction()?;
            if instruction == OpCode::OP_RETURN {
                return Ok(());
            }

            match instruction {
                OpCode::OP_CONSTANT => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::OP_NEGATE => {
                    let constant = (-self.pop().clone())?;
                    self.push(constant);
                }
                OpCode::OP_TRUE => self.push(Value::VAL_BOOL(true)),
                OpCode::OP_FALSE => self.push(Value::VAL_BOOL(false)),
                OpCode::OP_EQUAL => {
                    let b = self.pop().clone();
                    let a = self.pop().clone();
                    let equal = self.values_equal(a, b);
                    self.push(Value::VAL_BOOL(equal));
                }
                OpCode::OP_GREATER => self.binary_op(BinaryOp::Greater),
                OpCode::OP_LESS => self.binary_op(BinaryOp::Less),
                OpCode::OP_NIL => self.push(Value::VAL_NIL),
                OpCode::OP_NOT => {
                    let val = self.pop().clone();
                    self.push(Value::VAL_BOOL(self.is_falsey(val)))
                }
                OpCode::OP_ADD => {
                    if let Some(a) = self.peek_at(0) {
                        if let Some(b) = self.peek_at(1) {
                            match (a, b) {
                                (Value::VAL_OBJECT(oa), Value::VAL_OBJECT(ob)) => {
                                    if let ObjectType::OBJ_STRING(a) = &oa.object_type {
                                        if let ObjectType::OBJ_STRING(b) = &oa.object_type {
                                            self.concatenate()
                                        }
                                    }
                                }
                                _ => self.runtime_error(anyhow!(
                                    "Operands must be either addable or concatenatable."
                                )),
                            }
                        }
                    }
                    self.binary_op(BinaryOp::Add)
                }
                OpCode::OP_SUBTRACT => self.binary_op(BinaryOp::Sub),
                OpCode::OP_MULTIPLY => self.binary_op(BinaryOp::Mul),
                OpCode::OP_DIVIDE => self.binary_op(BinaryOp::Div),
                OpCode::OP_RETURN => {
                    return Ok(());
                }
            }
        }
    }

    fn read_byte(&mut self) -> Instruction {
        let instruction = self.chunk.code[self.ip];
        self.ip += 1;
        instruction
    }

    fn read_instruction(&mut self) -> Result<OpCode> {
        let instruction = self.read_byte();
        let op_code = OpCode::try_from(instruction)?;
        Ok(op_code)
    }

    fn read_constant(&mut self) -> Value {
        let instruction = self.read_byte();
        self.chunk.constants.values[instruction as usize].clone()
    }

    fn binary_op(&mut self, op: BinaryOp) {
        let b = self.pop().clone();
        let a = self.pop().clone();
        let val = match op {
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Div => a / b,
            BinaryOp::Mul => a * b,
            BinaryOp::Greater => {
                self.push(Value::VAL_BOOL(a > b));
                return;
            }
            BinaryOp::Less => {
                self.push(Value::VAL_BOOL(a < b));
                return;
            }
        };
        match val {
            Ok(val) => self.push(Value::VAL_NUMBER(val)),
            Err(e) => self.runtime_error(e),
        }
    }

    fn runtime_error(&self, error: anyhow::Error) {
        eprintln!("{error}");

        let instruction = self.ip - self.sp - 1;
        let line = self.chunk.lines[instruction];

        eprintln!("[line {line}] in script");
    }

    fn is_falsey(&self, value: Value) -> bool {
        value == Value::VAL_NIL || value == Value::VAL_BOOL(false)
    }
    fn values_equal(&self, a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::VAL_BOOL(a), Value::VAL_BOOL(b)) => a == b,
            (Value::VAL_NUMBER(a), Value::VAL_NUMBER(b)) => a == b,
            (Value::VAL_OBJECT(oa), Value::VAL_OBJECT(ob)) => oa == ob,
            (Value::VAL_NIL, Value::VAL_NIL) => true,
            _ => false,
        }
    }
    fn init_stack() -> [Option<Value>; MAX_STACK_SIZE] {
        const STACK_INIT: Option<Value> = None;
        [STACK_INIT; MAX_STACK_SIZE]
    }
    fn peek_at(&self, at: usize) -> &Option<Value> {
        &self.stack[self.sp - at]
    }
    fn concatenate(&mut self) {
        let b = self.pop().clone();
        let a = self.pop().clone();

        match (&a, &b) {
            (Value::VAL_OBJECT(oa), Value::VAL_OBJECT(ob)) => {
                match (oa.clone().object_type, ob.clone().object_type) {
                    (ObjectType::OBJ_STRING(mut a), ObjectType::OBJ_STRING(b)) => a.push_str(&b),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }

        let object = Object {
            object_type: ObjectType::OBJ_STRING(a.to_string()),
        };
        self.push(Value::VAL_OBJECT(object))
    }
}

#[derive(Error, Debug)]
pub enum InterpretError {
    #[error("runtime error")]
    RUNTIME_ERROR,
    #[error("compilation error")]
    COMPILE_ERROR,
}

#[cfg(test)]
mod tests {
    use crate::op_code::OpCode::*;

    use super::*;

    #[test]
    fn binary_operands_should_succeed() {
        let mut chunk = Chunk::default();

        let mut constant_index = chunk.add_constant(Value::VAL_NUMBER(1.1));
        chunk.write(OP_CONSTANT.into(), 123);
        chunk.write(constant_index as u8, 123);

        constant_index = chunk.add_constant(Value::VAL_NUMBER(3.3));
        chunk.write(OP_CONSTANT.into(), 123);
        chunk.write(constant_index as u8, 123);

        chunk.write(OP_ADD.into(), 123); // 1.1 + 3.3 = 4.4

        let constant_index = chunk.add_constant(Value::VAL_NUMBER(2.));
        chunk.write(OP_CONSTANT.into(), 123);
        chunk.write(constant_index as u8, 123);

        chunk.write(OP_DIVIDE.into(), 123); // 4.4 / 2.0 = 2.2
        chunk.write(OP_NEGATE.into(), 123); // - 2.2

        chunk.write(OP_RETURN.into(), 123);

        let stack = VM::init_stack();
        let mut vm = VM {
            chunk,
            ip: 0,
            stack,
            sp: 0,
        };

        vm.run();

        assert_eq!(vm.stack[0], Some(Value::VAL_NUMBER(-2.2)));
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.ip, 10);
    }
}
