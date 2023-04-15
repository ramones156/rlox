use crate::token::Token;

pub struct Parser {
    pub(crate) current: Option<Token>,
    pub(crate) previous: Option<Token>,
    pub(crate) had_error: bool,
    pub(crate) panic_mode: bool,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            current: None,
            previous: None,
            had_error: false,
            panic_mode: true,
        }
    }
}
