// This file uses modified code from https://github.com/obiesie/sox
// Copyright (c) 2023 obiesie
// parser

use crate::ast::{Command, Program, Redirect, Statement};
use crate::lexer::{Token, TokenType};
use std::iter::Peekable;

pub static TO_IGNORE: &'static [TokenType] =
    &[TokenType::Whitespace, TokenType::Lparen, TokenType::Rparen];

// TODO: proper parsing errors with line numbers.

// TODO: How does Peekable actually work and what is the peek method actually doing here?
pub struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
    processed_tokens: Vec<Token>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            processed_tokens: vec![],
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut program = Program::new();
        while !self.at_end() {
            let stmt = self.statement();
            if let Ok(val) = stmt {
                program.statements.push(val);
            } else if let Err(e) = stmt {
                return Err(e);
            }
        }

        return Ok(program);
    }

    // consumes token if match with any types in token_types
    fn match_token(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn skip_ignored(&mut self) {
        while let Some(t) = self.tokens.peek() {
            if TO_IGNORE.contains(&t.token_type) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        self.skip_ignored();
        matches!(self.tokens.peek(), Some(t) if t.token_type == token_type)
    }

    fn statement(&mut self) -> Result<Statement, String> {
        let mut statement = Statement::new();
        while !self.at_end() {
            // this should consume the command
            // add commands to the pipeline if there are multiple
            // A command must start with either word, stirng or Redirects.
            let command = self.command();
            if let Ok(val) = command {
                statement.pipeline.commands.push(val);
            } else {
                if let Err(e) = command {
                    return Err(e);
                }
            }

            if self.match_token(vec![TokenType::Nl, TokenType::Semicolon]) {
                return Ok(statement);
            }
            self.match_token(vec![TokenType::Pipe]);
        }

        return Ok(statement);
    }

    // what do we care about?
    // - word
    // - string
    // - semicolon
    // - newline
    // - async
    // - redirectOut / redirectIn / redirectOutAppend - last redirect is what counts
    // - Pipeline
    //
    // while peek token is not terminator, add to argv
    fn command(&mut self) -> Result<Command, String> {
        let mut command = Command::new();
        while let Some(peek_val) = self.tokens.peek() {
            match peek_val.token_type {
                TokenType::Pipe | TokenType::Semicolon | TokenType::Nl | TokenType::Async => break,
                TokenType::Word | TokenType::StringLiteral => {
                    command.argv.push(peek_val.lexeme.clone());
                    self.advance();
                }
                TokenType::RedirectOut => {
                    self.advance();
                    if self.at_end() {
                        return Err("Empty output file".to_string());
                    }
                    let val = self.match_token(vec![TokenType::Word, TokenType::StringLiteral]);
                    let tok = self.processed_tokens.last();
                    if !val {
                        return Err("Syntax error near RedirectOut".to_string());
                    } else if let Some(tok) = tok {
                        command.redirect = Redirect::Out(tok.lexeme.clone());
                    }
                }
                TokenType::RedirectOutAppend => {
                    self.advance();
                    if self.at_end() {
                        return Err("Empty output file".to_string());
                    }
                    let val = self.match_token(vec![TokenType::Word, TokenType::StringLiteral]);
                    let tok = self.processed_tokens.last();
                    if !val {
                        return Err("Syntax error near RedirectOut".to_string());
                    } else if let Some(tok) = tok {
                        command.redirect = Redirect::OutAppend(tok.lexeme.clone());
                    }
                }
                TokenType::RedirectIn => {
                    if command.argv.is_empty() {
                        return Err("Empty command list and redirect in".to_string());
                    }
                    self.advance();
                    if self.at_end() {
                        return Err("Empty input file".to_string());
                    }
                    let val = self.match_token(vec![TokenType::Word, TokenType::StringLiteral]);
                    let tok = self.processed_tokens.last();
                    if !val {
                        return Err("Syntax error near RedirectOut".to_string());
                    } else if let Some(tok) = tok {
                        command.redirect = Redirect::In(tok.lexeme.clone());
                    }
                }
                TokenType::Whitespace | TokenType::Lparen | TokenType::Rparen => {
                    self.advance();
                }
                _ => {}
            }
        }

        Ok(command)
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.at_end() {
            let token = self.tokens.next();
            let ret = token.unwrap();
            self.processed_tokens.push(ret.clone());
            return Some(ret);
        }

        None
    }

    fn at_end(&mut self) -> bool {
        matches!(
            self.tokens.peek(),
            Some(Token {
                token_type: TokenType::Eof,
                ..
            }) | None
        )
    }
}
