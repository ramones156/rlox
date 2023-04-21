use std::iter::Peekable;
use std::slice;
use std::slice::Iter;
use std::thread::{current, scope};

use crate::token::TokenType::*;
use crate::token::{Token, TokenType};

type PeekableToken<'a> = Peekable<slice::Iter<'a, &'a u8>>;

pub struct Scanner {
    pub(crate) source: Vec<u8>,
    start: usize,
    pub(crate) current: usize,
    line: usize,
    is_finished: bool,
}

impl Scanner {
    pub fn new(source: Vec<u8>) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            is_finished: false,
        }
    }

    pub fn scan_token(&mut self) -> Option<Token> {
        self.start = self.current;

        let source = self.source.clone();
        let current_token = source.iter().skip(self.start).collect::<Vec<_>>();

        let mut current_token = current_token.iter().peekable();

        self.skip_whitespace(&mut current_token);

        if self.is_finished {
            return None;
        }

        if self.is_at_end(&mut current_token) {
            self.is_finished = true;
            return Some(self.make_token(TOKEN_EOF));
        }

        if let Some(&c) = current_token.peek() {
            self.start = self.current;
            if Self::is_digit(c) {
                return Some(self.number(&mut current_token));
            };
            if Self::is_alpha(c) {
                return Some(self.identifier(&mut current_token));
            };
            self.advance();
            current_token.next();
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
                '"' => return Some(self.string(&mut current_token)),
                _ => return Some(self.error_token("Unexpected character.")),
            };

            let token = self.make_token(token_type);

            return Some(token);
        }

        None
    }

    fn advance(&mut self) {
        self.current += 1;
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
        self.current == self.source.len()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        let message = self.source[self.start..self.current].to_vec();
        let message = String::from_utf8(message).unwrap();
        Token::new(token_type, message, self.start, self.line)
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            token_type: TOKEN_ERROR,
            message: message.to_string(),
            start: self.start,
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self, token: &mut PeekableToken) {
        while let Some(&c) = token.peek() {
            match **c as char {
                ' ' | '\r' | '\t' => {
                    self.advance();
                    self.start = self.current;
                    token.next();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                    self.start = self.current;
                    token.next();
                }
                '/' => {
                    self.advance();
                    token.next();
                    if let Some(&next) = token.peek() {
                        self.advance();
                        token.next();
                        if **next as char == '/' {
                            while let Some(&next) = token.peek() {
                                if **next as char != '\n' {
                                    self.advance();
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
        loop {
            if let Some(&c) = token.peek() {
                self.advance();
                token.next();
                if **c as char == '"' {
                    break;
                }
                if **c as char == '\n' {
                    self.line += 1;
                }
            } else {
                return self.error_token("Unterminated string.");
            }
        }

        self.make_token(TOKEN_STRING)
    }

    fn number(&mut self, token: &mut PeekableToken) -> Token {
        loop {
            if let Some(&c) = token.peek() {
                if Self::is_digit(c) {
                    self.advance();
                    token.next();
                    continue;
                }
            }
            break;
        }
        if let Some(&c) = token.peek() {
            token.next();
            if let Some(&c2) = token.peek() {
                if **c as char == '.' && Self::is_digit(c2) {
                    self.advance();
                    loop {
                        if let Some(&c) = token.peek() {
                            if Self::is_digit(c) {
                                self.advance();
                                token.next();
                                continue;
                            }
                        }
                        break;
                    }
                }
            }
        }
        self.make_token(TOKEN_NUMBER)
    }

    fn identifier(&mut self, token: &mut PeekableToken) -> Token {
        let token = self.identifier_type(token);
        self.make_token(token)
    }

    fn identifier_type(&mut self, token: &mut PeekableToken) -> TokenType {
        if let Some(&c) = token.peek() {
            match **c as char {
                'a' => return self.check_keyword(1, "nd", TOKEN_AND),
                'c' => return self.check_keyword(1, "lass", TOKEN_CLASS),
                'e' => return self.check_keyword(1, "lse", TOKEN_ELSE),
                'f' => {
                    self.advance();
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
                'i' => return self.check_keyword(1, "f", TOKEN_IF),
                'n' => return self.check_keyword(1, "il", TOKEN_NIL),
                'o' => return self.check_keyword(1, "r", TOKEN_OR),
                'p' => return self.check_keyword(1, "rint", TOKEN_PRINT),
                'r' => return self.check_keyword(1, "eturn", TOKEN_RETURN),
                's' => return self.check_keyword(1, "uper", TOKEN_SUPER),
                'v' => return self.check_keyword(1, "ar", TOKEN_VAR),
                'w' => return self.check_keyword(1, "hile", TOKEN_WHILE),
                't' => {
                    self.advance();
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
        self.advance();
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
        self.current = right;

        let possible_rest = String::from_utf8(self.source[left..right].to_vec()).unwrap();
        if &*possible_rest == rest {
            return token_type;
        }

        TOKEN_IDENTIFIER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_token(
        scanner: &mut Scanner,
        token_type: TokenType,
        token_message: &str,
        start_buffer: usize,
        line: usize,
    ) {
        let token = scanner.scan_token();
        assert!(token.is_some());

        let token = token.unwrap();
        assert_eq!(token.token_type, token_type);
        assert_eq!(token.message, token_message.to_string());
        assert_eq!(token.start, start_buffer);
        assert_eq!(token.line, line);
    }

    #[test]
    fn assign_should_succeed() {
        let source = "var x = 5".to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_VAR, "var", 0, 1);
        assert_token(&mut scanner, TokenType::TOKEN_IDENTIFIER, "x", 4, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EQUAL, "=", 6, 1);
        assert_token(&mut scanner, TokenType::TOKEN_NUMBER, "5", 8, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EOF, "", 9, 1);
    }

    #[test]
    fn string_should_succeed() {
        let source = r#"var x = "string""#.to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_VAR, "var", 0, 1);
        assert_token(&mut scanner, TokenType::TOKEN_IDENTIFIER, "x", 4, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EQUAL, "=", 6, 1);
        assert_token(&mut scanner, TokenType::TOKEN_STRING, r#""string""#, 8, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EOF, "", 16, 1);
    }

    #[test]
    fn boolean_should_succeed() {
        let source = "true".to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_TRUE, "true", 0, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EOF, "", 4, 1);

        let source = "false".to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_FALSE, "false", 0, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EOF, "", 5, 1);

        let source = "!false".to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_BANG, "!", 0, 1);
        assert_token(&mut scanner, TokenType::TOKEN_FALSE, "false", 1, 1);
        assert_token(&mut scanner, TokenType::TOKEN_EOF, "", 6, 1);
    }

    #[test]
    fn print_should_succeed() {
        let source = "print 5".to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_PRINT, "print", 0, 1);
        assert_token(&mut scanner, TokenType::TOKEN_NUMBER, "5", 6, 1);
    }

    #[test]
    fn newline_should_succeed() {
        let source = "\n3".to_string().into_bytes();
        let mut scanner = Scanner::new(source);

        assert_token(&mut scanner, TokenType::TOKEN_NUMBER, "3", 1, 2);
    }
}
