// parser

use core::panic;

use crate::{
    ast::{Command, Commands, Program, Statement},
    lexer::{Lexer, Token, TokenType},
};

pub struct Parser<'source> {
    lex: Lexer<'source>,
    ast: Program,
    curr: Token,
    next: Token,
}

impl<'source> Parser<'source> {
    pub fn parser(text: &'source str) -> Self {
        let lexer = Lexer::lex(text);
        let parse = Self {
            lex: lexer,
            ast: Program::new(),
            curr: Token::empty_token(),
            next: Token::empty_token(),
        };
        parse
    }

    pub fn parse_program(&mut self) {
        if let Some(tok) = self.lex.next() {
            self.curr = tok;
        }
        self.parse_statements();
    }
    pub fn parse_statements(&mut self) {
        while let Some(statement) = self.parse_statement() {
            self.ast.statements.push(statement);
        }
    }
    pub fn parse_statement(&mut self) -> Option<Statement> {}
    pub fn parse_pipeline(&mut self) -> Commands {}
    pub fn parse_command(&mut self) -> Option<Command> {
        let last_redirect = false;
        let command = Command::new();
        while let Some(tok) = self.lex.next()
            && tok.token_type != TokenType::Eof
        {
            self.curr = self.next;
            self.next = tok;
            match self.curr.token_type {
                TokenType::Whitespace => continue,
                TokenType::RedirectOut => {
                }
                TokenType::RedirectOutAppend =>
                TokenType::RedirectIn =>
                TokenType::Pipe =>
                TokenType::Semicolon =>
                TokenType::Async =>
                TokenType::Lparen =>
                TokenType::Rparen =>
                TokenType::Nl =>

                // suitable for argv
                TokenType::StringLiteral => command.argv.push(tok.literal),
                TokenType::Word => command.argv.push(tok.literal)
                // TODO: what can this be?
                _ => panic!("Error?"),
            }
        }

        Some(Command::new())
    }
}
