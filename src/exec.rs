// exec

// TODO: investigate libc vs unistd versions of the same functions
// TODO: What type should Error be? How to return the error code from a execvp function?
// TODO: convert other iterative code to more rust idiomatic code.
// TODO: Fix warnings
// TODO: Rust method vs function
use nix::fcntl::OFlag;
use nix::fcntl::open;
use nix::sys::stat::Mode;
use nix::sys::wait::waitpid;
use nix::unistd::ForkResult;
use nix::unistd::close;
use nix::unistd::dup2_stdin;
use nix::unistd::dup2_stdout;
use nix::unistd::execvp;
use nix::unistd::fork;
use nix::unistd::pipe;
use std::ffi::CString;
use std::os::fd::AsRawFd;
use std::os::fd::OwnedFd;

use crate::ast::Redirect;
use crate::ast::{Program, Statement};

pub struct Executor {
    return_code: u64,
}

impl Executor {
    pub fn new() -> Self {
        Self { return_code: 0 }
    }

    pub fn exec(&self, ast: &Program) -> u64 {
        let statements = &ast.statements;
        statements.iter().for_each(|stmt| {
            exec_statement(stmt);
        });
        return 0;
    }
}

fn apply_redirect(r: &Redirect) -> Result<(), nix::Error> {
    match r {
        Redirect::In(path) => {
            let path = CString::new(path.clone()).unwrap();
            let fd = open(path.as_c_str(), OFlag::O_RDONLY, Mode::S_IRUSR).unwrap();
            dup2_stdin(&fd)?;
            close(fd)?;
            Ok(())
        }
        Redirect::Out(path) => {
            let path = CString::new(path.clone()).unwrap();
            let fd = open(
                path.as_c_str(),
                OFlag::O_WRONLY | OFlag::O_CREAT,
                Mode::S_IWUSR | Mode::S_IRUSR,
            )
            .unwrap();
            dup2_stdout(&fd)?;
            close(fd)?;
            Ok(())
        }
        Redirect::OutAppend(path) => {
            let path = CString::new(path.clone()).unwrap();
            let fd = open(
                path.as_c_str(),
                OFlag::O_APPEND | OFlag::O_CREAT,
                Mode::S_IWUSR | Mode::S_IRUSR,
            )
            .unwrap();
            dup2_stdout(&fd)?;
            close(fd)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn exec_statement(stmt: &Statement) -> Result<(), nix::Error> {
    let mut prev_read: Option<OwnedFd> = None;
    let mut children = Vec::new();
    for (i, cmd) in stmt.pipeline.commands.iter().enumerate() {
        let (read, write) = if i < stmt.pipeline.commands.len() - 1 {
            let (r, w) = pipe()?;
            (Some(r), Some(w))
        } else {
            (None, None)
        };

        match unsafe { fork()? } {
            ForkResult::Child => {
                if let Some(fd) = &prev_read {
                    dup2_stdin(fd)?;
                    close(fd.as_raw_fd())?;
                }
                if let Some(fd) = write {
                    dup2_stdout(&fd)?;
                    close(fd)?;
                }
                if let Some(r) = read {
                    close(r)?;
                }
                let redirect = &cmd.redirect;
                apply_redirect(redirect)?;

                let cstr_argv: Vec<CString> = cmd
                    .argv
                    .iter()
                    .map(|arg| CString::new(arg.as_str()).unwrap())
                    .collect();
                let file = &cstr_argv[0];
                let argv = &cstr_argv;

                let Err(e) = execvp(file, argv);
                eprintln!("exec failed {}", e);
                std::process::exit(1);
            }
            ForkResult::Parent { child, .. } => {
                children.push(child);
                if let Some(fd) = prev_read {
                    close(fd)?;
                }
                if let Some(w) = write {
                    close(w)?;
                }
                prev_read = read;
            }
        }
    }

    for pid in children {
        waitpid(pid, None);
    }
    Ok(())
}
