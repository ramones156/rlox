#![allow(unused)]

use anyhow::Result;
use rlox::chunk::Chunk;
use rlox::compiler::Compiler;
use rlox::op_code::OpCode::{OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_NEGATE, OP_RETURN};
use rlox::vm::{InterpretError, VM};
use std::io::{BufRead, Write};
use std::process::exit;

fn main() {
    let mut chunk = Chunk::default();
    let mut interpreter = Interpreter {
        compiler: Compiler::new(&mut chunk),
    };
    let args = std::env::args();
    let argc = args.len();
    if argc == 1 {
        interpreter.repl();
    } else if argc == 2 {
        let code = args.collect::<Vec<_>>()[1].clone();
        interpreter.run_file(code);
    } else {
        eprintln!("Usage: rlox: [path]");
        exit(64);
    }
}

struct Interpreter<'a> {
    compiler: Compiler<'a>,
}

impl<'a> Interpreter<'a> {
    fn repl(&mut self) {
        let mut buffer = String::new();
        loop {
            std::io::stdout().write_all("> ".as_bytes()).unwrap();
            std::io::stdout().flush().unwrap();

            let _ = std::io::stdin().lock().read_line(&mut buffer).unwrap();
            self.interpret(buffer.clone().into_bytes()).unwrap();

            buffer.clear();
        }
    }

    fn interpret(&mut self, source: Vec<u8>) -> Result<(), InterpretError> {
        self.compiler.compile(source);
        Ok(())
    }

    fn run_file(&mut self, path: String) -> Result<()> {
        let source = Self::read_file(path)?;

        match self.interpret(source) {
            Ok(_) => {}
            Err(e) => match e {
                InterpretError::COMPILE_ERROR => exit(65),
                InterpretError::RUNTIME_ERROR => exit(70),
            },
        }

        Ok(())
    }

    fn read_file(path: String) -> Result<Vec<u8>> {
        let file = std::fs::read(path)?;
        Ok(file)
    }
}
