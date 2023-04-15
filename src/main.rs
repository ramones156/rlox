#![allow(unused)]

use anyhow::Result;
use rlox::chunk::Chunk;
use rlox::compiler::Compiler;
use rlox::op_code::OpCode::{OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_NEGATE, OP_RETURN};
use rlox::vm::{InterpretError, VM};
use std::io::{BufRead, Write};
use std::process::exit;

fn main() {
    let args = std::env::args();
    let argc = args.len();
    if argc == 1 {
        repl();
    } else if argc == 2 {
        let code = args.collect::<Vec<_>>()[1].clone();
        run_file(code);
    } else {
        eprintln!("Usage: rlox: [path]");
        exit(64);
    }
}

fn repl() {
    let mut buffer = String::new();
    loop {
        std::io::stdout().write_all("> ".as_bytes()).unwrap();
        std::io::stdout().flush().unwrap();

        let _ = std::io::stdin().lock().read_line(&mut buffer).unwrap();
        buffer = buffer.trim().to_string();
        interpret(buffer.clone().into_bytes()).unwrap();

        buffer.clear();
    }
}

fn interpret(source: Vec<u8>) -> Result<(), InterpretError> {
    let mut chunk = Chunk::default();
    let mut compiler = Compiler::new(&mut chunk);
    compiler.compile(source);
    Ok(())
}

fn run_file(path: String) -> Result<()> {
    let source = read_file(path)?;

    match interpret(source) {
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
