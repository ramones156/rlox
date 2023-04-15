use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum OpCode {
    OP_CONSTANT,
    OP_ADD,
    OP_SUBTRACT,
    OP_MULTIPLY,
    OP_DIVIDE,
    OP_NEGATE,
    OP_RETURN,
}
