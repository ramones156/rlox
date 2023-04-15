use crate::chunk::{Chunk, Instruction};
use crate::compiler::Compiler;
use crate::op::BinaryOp;
use crate::op_code::OpCode;
use crate::value::Value;
use crate::vm::InterpretError::COMPILE_ERROR;
use anyhow::Result;
use num_enum::TryFromPrimitiveError;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use thiserror::Error;

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

        let mut vm = Self {
            chunk,
            ip: 0,
            stack: [None; MAX_STACK_SIZE],
            sp: 0,
        };

        vm.run()?;
        Ok(())
    }

    fn push(&mut self, value: Value) {
        self.stack[self.sp] = Some(value);
        self.sp += 1;
    }

    fn pop(&mut self) -> Value {
        self.sp -= 1;
        self.stack[self.sp].unwrap()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            print!("        ");
            for i in 0..self.sp {
                print!("[ ");
                print!("{:?}", self.stack[i].unwrap());
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
                    let constant = -self.pop();
                    self.push(constant);
                }
                OpCode::OP_ADD => self.binary_op(BinaryOp::Add),
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
        self.chunk.constants.values[instruction as usize]
    }

    fn binary_op(&mut self, op: BinaryOp) {
        let b = self.pop();
        let a = self.pop();
        let val = match op {
            BinaryOp::Add => a + b,
            BinaryOp::Sub => a - b,
            BinaryOp::Div => a / b,
            BinaryOp::Mul => a * b,
        };
        self.push(val)
    }
}

#[derive(Error, Debug)]
#[allow(non_camel_case_types)]
pub enum InterpretError {
    #[error("runtime error")]
    RUNTIME_ERROR,
    #[error("compilation error")]
    COMPILE_ERROR,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op_code::OpCode::*;

    #[test]
    fn should_succeed() {
        let mut chunk = Chunk::new();

        let mut constant_index = chunk.add_constant(1.1);
        chunk.write(OP_CONSTANT.into(), 123);
        chunk.write(constant_index as u8, 123);

        constant_index = chunk.add_constant(3.3);
        chunk.write(OP_CONSTANT.into(), 123);
        chunk.write(constant_index as u8, 123);

        chunk.write(OP_ADD.into(), 123); // 1.1 + 3.3 = 4.4

        let constant_index = chunk.add_constant(2.);
        chunk.write(OP_CONSTANT.into(), 123);
        chunk.write(constant_index as u8, 123);

        chunk.write(OP_DIVIDE.into(), 123); // 4.4 / 2.0 = 2.2
        chunk.write(OP_NEGATE.into(), 123); // - 2.2

        chunk.write(OP_RETURN.into(), 123);

        let mut vm = VM {
            chunk,
            ip: 0,
            stack: [None; MAX_STACK_SIZE],
            sp: 0,
        };

        vm.run();

        assert_eq!(vm.stack[0], Some(-2.2));
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.ip, 10);
    }
}
