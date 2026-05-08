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
//
// We also support pipes:
// git log --oneline --author=rajeshrah22 | wc -l
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct Statement {
    pub pipeline: Commands,
    pub background: bool,
}

#[derive(Debug)]
pub struct Commands {
    pub commands: Vec<Command>,
}

// TODO: or Redirects?
#[derive(Debug)]
pub struct Command {
    pub argv: Vec<String>,
    pub redirect: Redirect,
}

#[derive(Debug)]
pub enum Redirect {
    In(String),
    Out(String),
    OutAppend(String),
    None,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

impl Statement {
    pub fn new() -> Self {
        Self {
            pipeline: Commands::new(),
        }
    }
}

impl Commands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl Command {
    pub fn new() -> Self {
        Self {
            argv: Vec::new(),
            // TODO: Is a none type in enum bad practice?
            // Should I make redirect Option<Redirect> type instead?
            redirect: Redirect::None,
        }
    }
}
