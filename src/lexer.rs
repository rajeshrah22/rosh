// This file uses modified code from https://github.com/obiesie/sox
// Copyright (c) 2023 obiesie

use std::ops::Range;

// TODO: PartialEq vs Eq

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Word,
    StringLiteral,
    Nl,
    Whitespace,
    Eof,
    Pipe,
    RedirectOut,
    RedirectOutAppend,
    RedirectIn,
    Semicolon,
    Async,
    Lparen,
    Rparen,
    Err,
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl Token {
    pub fn empty_token() -> Self {
        Self {
            token_type: TokenType::None,
            lexeme: "".to_string(),
            literal: Literal::None,
            line: 0,
        }
    }
}

pub struct Lexer<'source> {
    source: &'source str,
    start: usize,
    curr: usize,
    line: usize,
}

impl<'source> Lexer<'source> {
    fn new(source: &'source str) -> Self {
        return Self {
            source,
            start: 0,
            curr: 0,
            line: 0,
        };
    }

    pub fn lex(text: &'source str) -> Self {
        let lexer = Self::new(text);
        lexer
    }

    fn is_at_end(&self) -> bool {
        let len = self.source.len();
        return self.curr >= len;
    }

    // take current character and move by one
    fn advance(&mut self) -> Option<char> {
        let curr_char = self.source.chars().nth(self.curr);
        self.curr += 1;
        return curr_char;
    }

    // look at next char
    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.curr)
    }

    // TODO
    fn yield_whitespace(&mut self) -> Token {
        self.take_while(|c| c == ' ' || c == '\t');
        Token {
            token_type: TokenType::Whitespace,
            lexeme: "".to_string(),
            literal: Literal::None,
            line: self.line,
        }
    }

    fn yield_literal_token(&self, token_type: TokenType, literal: Literal) -> Token {
        let text = self.source.get(self.start..self.curr).unwrap_or("");
        Token {
            token_type: token_type,
            lexeme: text.to_string(),
            literal: literal,
            line: self.line,
        }
    }

    fn yield_token(&self, token_type: TokenType) -> Token {
        self.yield_literal_token(token_type, Literal::None)
    }

    fn char_matches(&mut self, c: char) -> bool {
        if self.peek().unwrap_or('\0') != c {
            return false;
        }

        self.curr += 1;
        return true;
    }

    // also increments line counter
    fn take_while<P>(&mut self, mut predicate: P) -> Option<(&'source str, Range<usize>)>
    where
        P: FnMut(char) -> bool,
    {
        let start = self.start;

        while let Some(c) = self.peek() {
            if !predicate(c) {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        let end = self.curr;

        if start != end {
            let text = &self.source[start..end];
            Some((text, start..end))
        } else {
            None
        }
    }

    // Strings can span multiple lines
    fn yield_string(&mut self, delim: char) -> Result<Token, String> {
        let value = self.take_while(|c| c != delim);
        self.advance();

        if let Some((str_literal, _)) = value {
            if self.is_at_end() && self.source.chars().last().unwrap() != delim {
                panic!("unterminated string");
            }
            let token = self.yield_literal_token(
                TokenType::StringLiteral,
                Literal::String(str_literal[1..].to_string()),
            );
            Ok(token)
        } else {
            Err("".into())
        }
    }

    // TODO
    fn yield_word(&mut self) -> Result<Token, String> {
        let p = |c: char| match c {
            '(' | ')' | '<' | '>' | ';' | '|' | '\n' | ' ' | '\t' => false,
            _ => true,
        };
        let value = self.take_while(p);
        if let Some((str_literal, _)) = value {
            let token =
                self.yield_literal_token(TokenType::Word, Literal::String(str_literal.to_string()));
            Ok(token)
        } else {
            Err("".into())
        }
    }

    fn token_from_result(&self, input: Result<Token, String>) -> Option<Token> {
        match input {
            Ok(v) => Some(v),
            Err(e) => Some(Token {
                token_type: TokenType::Err,
                lexeme: e,
                literal: Literal::None,
                line: self.line,
            }),
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_at_end() {
            self.start = self.curr;
            let character = self.advance();
            let token = if let Some(character) = character {
                match character {
                    // special characters
                    '(' => Some(self.yield_token(TokenType::Lparen)),
                    ')' => Some(self.yield_token(TokenType::Rparen)),
                    '<' => Some(self.yield_token(TokenType::RedirectIn)),
                    '>' => {
                        if self.char_matches(character) {
                            Some(self.yield_token(TokenType::RedirectOutAppend))
                        } else {
                            Some(self.yield_token(TokenType::RedirectOut))
                        }
                    }
                    '&' => Some(self.yield_token(TokenType::Async)),
                    ';' => Some(self.yield_token(TokenType::Semicolon)),
                    '|' => Some(self.yield_token(TokenType::Pipe)),
                    '\n' => {
                        let newline_token = self.yield_token(TokenType::Nl);
                        self.line += 1;
                        Some(newline_token)
                    }
                    '"' => {
                        let res = self.yield_string(character);
                        self.token_from_result(res)
                    }
                    '\'' => {
                        let res = self.yield_string(character);
                        self.token_from_result(res)
                    }
                    ' ' => Some(self.yield_whitespace()),
                    '\t' => Some(self.yield_whitespace()),

                    // word
                    _ => {
                        let res = self.yield_word();
                        self.token_from_result(res)
                    }
                }
            } else {
                Some(Token {
                    token_type: TokenType::Err,
                    lexeme: "".to_string(),
                    literal: Literal::None,
                    line: self.line,
                })
            };
            token
        } else {
            Some(Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                line: self.line,
            })
        }
    }
}
