use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Precedence {
    PREC_NONE,
    PREC_ASSIGNMENT, // =
    PREC_OR,         // or
    PREC_AND,        // and
    PREC_EQUALITY,   // == !=
    PREC_COMPARISON, // < > <= >=
    PREC_TERM,       // + -
    PREC_FACTOR,     // * /
    PREC_UNARY,      // ! -
    PREC_CALL,       // . ()
    PREC_PRIMARY,
}
