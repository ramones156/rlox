use std::ops::Sub;

use anyhow::Result;

use crate::op_code::OpCode;
use crate::value::{Value, ValueArray};

pub type Instruction = u8;

#[derive(Default)]
pub struct Chunk {
    pub(crate) code: Vec<Instruction>,
    count: usize,
    pub(crate) constants: ValueArray,
    pub(crate) lines: Vec<usize>,
}

impl Chunk {
    pub fn write(&mut self, data: u8, line: usize) {
        if self.code.len() < self.count + 1 {
            self.code.push(data);
            self.lines.push(line);
        } else {
            self.code[self.count] = data;
            self.lines[self.count] = line;
        }

        self.count += 1;
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.count - 1
    }

    pub fn disassemble_chunk(&self, name: String) -> Result<()> {
        println!("==== {name:<8}  ====");

        let mut offset = 0;
        while offset < self.count {
            offset = self.disassemble_instruction(offset)?;
        }

        Ok(())
    }

    pub(crate) fn disassemble_instruction(&self, offset: usize) -> Result<usize> {
        print!("{offset:04} ");
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let op_code = OpCode::try_from(self.code[offset])?;
        Ok(match op_code {
            OpCode::OP_CONSTANT => self.constant_instruction("OP_CONSTANT", offset),
            _ => Self::simple_instruction(&op_code, offset),
        })
    }

    fn simple_instruction(name: &OpCode, offset: usize) -> usize {
        println!("{name:?}");
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{name:-16} {constant:02} ");
        print!("{:?}", self.constants.values[constant as usize]);
        println!();
        offset + 2
    }
}
