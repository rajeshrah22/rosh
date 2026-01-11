use std::env;
use std::io;
use std::io::Write;

pub mod ast;
pub mod exec;
pub mod job;
pub mod lexer;
pub mod parser;

use exec::exec;
use exec::init_shell;
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

        if bytes_read == 1 {
            continue;
        }

        init_shell();

        let lex = Lexer::lex(buffer.as_str());
        let mut parse = Parser::new(lex);
        let ast = parse.parse();

        if ast.is_ok() {
            let ast = ast.unwrap();
            dbg!(&ast);
            exec(&ast);
        } else if let Err(e) = ast {
            println!("{}", e.as_str());
        }
    }
}
