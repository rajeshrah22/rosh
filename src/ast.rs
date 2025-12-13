// ast

// Conceptually this AST is a program, which is a series of statements.
// A statement is a pipeline of commands ended by newline or semicolon
// or ampersand.
//
// Let's think about some examples:
// - echo "hi" > log
// This one is a command with one output redirect.
//
// - cat < hello > log.txt
// This is an example with one input redirection from hello into cat and output redirection into
// log.txt
//
// - echo hello > a > b
// apparently, shells skip file a and output on last file.
//
// - echo hello >> a
// This appends output of `echo hello` into a.
//
// -- Series of commands;
// echo hello > a; echo rahul > b;
// 2 statements and redirections in one line.
pub struct Program {
    pub statements: Vec<Statement>,
}

pub struct Statement {
    pub pipeline: Commands,
}

pub struct Commands {
    pub commands: Vec<Command>,
}

pub struct Command {
    pub argv: Vec<String>,
    pub redirects: Vec<Redirect>,
}

pub enum Redirect {
    In(String),
    Out(String),
    OutAppend(String),
}
