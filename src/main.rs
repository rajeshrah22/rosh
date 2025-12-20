use std::env;
use std::io;
use std::io::Write;

pub mod ast;
pub mod exec;
pub mod lexer;
pub mod parser;

use exec::Executor;
use lexer::Lexer;
use parser::Parser;

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

        let lex = Lexer::lex(buffer.as_str());
        let mut parse = Parser::new(lex);
        let ast = parse.parse();
        let executor = Executor::new();
        if ast.is_ok() {
            let ast = ast.unwrap();
            dbg!(&ast);
            executor.exec(&ast);
        }

        if bytes_read == 1 {
            continue;
        }
    }
}
