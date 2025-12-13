use std::env;
use std::io;
use std::io::Write;

pub mod ast;
pub mod exec;
pub mod lexer;
pub mod parser;

use lexer::Lexer;

fn main() -> std::io::Result<()> {
    let cwd = env::current_dir()?;
    loop {
        print!("{}$ ", cwd.display());
        io::stdout().flush().unwrap();

        let mut buffer = String::new();
        let bytes_read = io::stdin().read_line(&mut buffer)?;

        if bytes_read == 0 {
            println!("\nexit");
            return Ok(());
        }

        let mut lex = Lexer::lex(buffer.as_str());

        while let Some(tok) = lex.next() {
            dbg!(tok);
        }

        if bytes_read == 1 {
            continue;
        }
    }
}
