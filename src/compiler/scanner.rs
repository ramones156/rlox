use crate::token::TokenType::*;
use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::slice;
use std::slice::Iter;
use std::thread::scope;

type PeekableToken<'a> = Peekable<slice::Iter<'a, &'a u8>>;

pub struct Scanner {
    pub(crate) source: Vec<u8>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: Vec<u8>) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Option<Token> {
        let source = self.source.clone();
        let current_token = source
            .iter()
            .skip(self.start)
            .take(self.current)
            .collect::<Vec<_>>();

        let mut token = current_token.iter().peekable();

        self.skip_whitespace(&mut token);

        if self.is_at_end(&mut token) {
            return Some(self.make_token(TOKEN_EOF));
        }

        if let Some(c) = token.next() {
            if Self::is_digit(c) {
                return Some(self.number(&mut token));
            };
            if Self::is_alpha(c) {
                return Some(self.identifier(&mut token));
            };
            let token_type = match **c as char {
                '(' => TOKEN_LEFT_PAREN,
                ')' => TOKEN_RIGHT_PAREN,
                '{' => TOKEN_LEFT_BRACE,
                '}' => TOKEN_RIGHT_BRACE,
                ';' => TOKEN_SEMICOLON,
                ',' => TOKEN_COMMA,
                '.' => TOKEN_DOT,
                '-' => TOKEN_MINUS,
                '+' => TOKEN_PLUS,
                '/' => TOKEN_SLASH,
                '*' => TOKEN_STAR,
                '!' => {
                    if self.match_token('-') {
                        TOKEN_BANG_EQUAL
                    } else {
                        TOKEN_BANG
                    }
                }
                '=' => {
                    if self.match_token('=') {
                        TOKEN_EQUAL_EQUAL
                    } else {
                        TOKEN_EQUAL
                    }
                }
                '<' => {
                    if self.match_token('=') {
                        TOKEN_LESS_EQUAL
                    } else {
                        TOKEN_LESS
                    }
                }
                '>' => {
                    if self.match_token('=') {
                        TOKEN_GREATER_EQUAL
                    } else {
                        TOKEN_GREATER
                    }
                }
                '"' => return Some(self.string(&mut token)),
                _ => return Some(self.error_token("Unexpected character.")),
            };
            let token = self.make_token(token_type);
            return Some(token);
        }

        None
    }

    fn match_token(&mut self, expected: char) -> bool {
        if let Some(&current) = self.source.get(self.current) {
            if current as char != expected {
                return false;
            }
        }

        self.current += 1;

        true
    }

    fn is_at_end(&self, token: &mut PeekableToken) -> bool {
        token.peek().is_none()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        dbg!(&token_type);
        let message = self.source[self.start..self.current].to_vec();
        let message = String::from_utf8(message).unwrap();
        Token::new(token_type, message, self.line)
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            token_type: TOKEN_ERROR,
            message: message.to_string(),
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self, token: &mut PeekableToken) {
        while let Some(&c) = token.peek() {
            match **c as char {
                ' ' | '\r' | '\t' => {
                    token.next();
                }
                '\n' => {
                    self.line += 1;
                    token.next();
                }
                '/' => {
                    token.next();
                    if let Some(&next) = token.peek() {
                        token.next();
                        if **next as char == '/' {
                            while let Some(&next) = token.peek() {
                                if **next as char != '\n' {
                                    token.next();
                                }
                            }
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn string(&mut self, token: &mut PeekableToken) -> Token {
        while let Some(&c) = token.peek() {
            if **c as char == '"' {
                break;
            }
            if **c as char == '\n' {
                self.line += 1;
            }
            token.next();
        }

        if token.next().is_none() {
            return self.error_token("Unterminated string.");
        }

        self.make_token(TOKEN_STRING)
    }

    fn number(&mut self, token: &mut PeekableToken) -> Token {
        while let Some(&c) = token.peek() {
            if Self::is_digit(c) {
                token.next();
            }
        }

        if let Some(&c) = token.peek() {
            token.next();
            if let Some(&c2) = token.peek() {
                if **c as char == '.' && Self::is_digit(c2) {
                    token.next();

                    while let Some(&c) = token.peek() {
                        if Self::is_digit(c) {
                            token.next();
                        }
                    }
                }
            }
        }

        self.make_token(TOKEN_NUMBER)
    }

    fn identifier(&mut self, token: &mut PeekableToken) -> Token {
        while let Some(&c) = token.peek() {
            if Self::is_alpha(c) || Self::is_digit(c) {
                token.next();
            }
        }

        let token = self.identifier_type(token);
        self.make_token(token)
    }

    fn identifier_type(&mut self, token: &mut PeekableToken) -> TokenType {
        if let Some(&c) = token.peek() {
            match **c as char {
                'a' => return self.check_keyword(1, "nd", TOKEN_AND),
                'c' => return self.check_keyword(1, "lass", TOKEN_CLASS),
                'e' => return self.check_keyword(1, "lse", TOKEN_ELSE),
                'i' => return self.check_keyword(1, "f", TOKEN_IF),
                'n' => return self.check_keyword(1, "il", TOKEN_NIL),
                'o' => return self.check_keyword(1, "r", TOKEN_OR),
                'p' => return self.check_keyword(1, "rint", TOKEN_PRINT),
                'r' => return self.check_keyword(1, "eturn", TOKEN_RETURN),
                's' => return self.check_keyword(1, "uper", TOKEN_SUPER),
                'v' => return self.check_keyword(1, "ar", TOKEN_VAR),
                'w' => return self.check_keyword(1, "hile", TOKEN_WHILE),
                'f' => {
                    token.next();
                    if let Some(&c) = token.peek() {
                        match **c as char {
                            'a' => return self.check_keyword(2, "lse", TOKEN_FALSE),
                            'o' => return self.check_keyword(2, "r", TOKEN_FOR),
                            'u' => return self.check_keyword(2, "n", TOKEN_FUN),
                            _ => {}
                        }
                    }
                }
                't' => {
                    token.next();
                    if let Some(&c) = token.peek() {
                        match **c as char {
                            'h' => return self.check_keyword(2, "is", TOKEN_THIS),
                            'r' => return self.check_keyword(2, "ue", TOKEN_TRUE),
                            _ => {}
                        }
                    }
                }
                _ => {}
            };
        }
        TOKEN_IDENTIFIER
    }

    fn is_digit(c: &u8) -> bool {
        (*c as char).is_ascii_digit()
    }

    fn is_alpha(c: &u8) -> bool {
        (*c as char).is_alphabetic()
    }

    fn check_keyword(&mut self, start: usize, rest: &str, token_type: TokenType) -> TokenType {
        let length = rest.len();
        let left = self.start + start;
        let right = left + length;

        let possible_rest = String::from_utf8(self.source[left..right].to_vec()).unwrap();
        if self.current - self.start == start + length && &*possible_rest == rest {
            return token_type;
        }
        TOKEN_IDENTIFIER
    }
}
