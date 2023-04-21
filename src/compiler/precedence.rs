use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, PartialEq, PartialOrd, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// token expected beforehand
pub enum Precedence {
    PREC_NONE,
    /// =
    PREC_ASSIGNMENT,
    /// or
    PREC_OR,
    /// and
    PREC_AND,
    /// == !=
    PREC_EQUALITY,
    /// < > <= >=
    PREC_COMPARISON,
    /// + -
    PREC_TERM,
    /// * /
    PREC_FACTOR,
    /// ! -
    PREC_UNARY,
    /// . ()
    PREC_CALL,
    PREC_PRIMARY,
}
