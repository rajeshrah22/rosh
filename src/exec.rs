// exec

// TODO: investigate libc vs unistd versions of the same functions
// TODO: What type should Error be? How to return the error code from a execvp function?
// TODO: convert other iterative code to more rust idiomatic code.
// TODO: Fix warnings
// TODO: Rust method vs function
// TODO: Set default sane terminal attributes somehow
use nix::fcntl::OFlag;
use nix::fcntl::open;
use nix::sys::signal::SigHandler;
use nix::sys::signal::Signal;
use nix::sys::signal::kill;
use nix::sys::signal::signal;
use nix::sys::stat::Mode;
use nix::sys::wait::waitpid;
use nix::unistd::ForkResult;
use nix::unistd::Pid;
use nix::unistd::close;
use nix::unistd::dup2_stdin;
use nix::unistd::dup2_stdout;
use nix::unistd::execvp;
use nix::unistd::fork;
use nix::unistd::getpgid;
use nix::unistd::getpgrp;
use nix::unistd::getpid;
use nix::unistd::pipe;
use nix::unistd::setpgid;
use nix::unistd::tcgetpgrp;
use nix::unistd::tcsetpgrp;
use std::ffi::CString;
use std::io::IsTerminal;
use std::io::stdin;
use std::os::fd::AsRawFd;
use std::os::fd::OwnedFd;

use crate::ast::Command;
use crate::ast::Redirect;
use crate::ast::{Program, Statement};

fn ignore_interactive_signals() {
    unsafe {
        signal(Signal::SIGSTOP, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGINT, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGQUIT, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGCHLD, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTOU, SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTIN, SigHandler::SigIgn).unwrap();
    };
}

fn default_signal_handlers() {
    unsafe {
        signal(Signal::SIGSTOP, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGINT, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGQUIT, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGCHLD, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGTTOU, SigHandler::SigDfl).unwrap();
        signal(Signal::SIGTTIN, SigHandler::SigDfl).unwrap();
    };
}

pub fn init_shell() {
    let is_interactive = stdin().is_terminal();

    if is_interactive {
        let mut term_pgid = { tcgetpgrp(stdin()) }.unwrap();
        let mut shell_pgid = getpgrp();

        // loop until we are in the foreground
        loop {
            if term_pgid == shell_pgid {
                break;
            }

            kill(shell_pgid, Signal::SIGTTIN).unwrap();
            term_pgid = tcgetpgrp(stdin()).unwrap();
            shell_pgid = getpgrp();
        }

        ignore_interactive_signals();

        shell_pgid = getpid();
        setpgid(shell_pgid, shell_pgid).unwrap();
        tcsetpgrp(stdin(), shell_pgid).unwrap();
    }
}

fn do_fg() {}
fn do_bg() {}

// 1. Put the process into the process group
//    and give process group the terminal if appropriate. This has to be done by both the shell and
//    appropriate child processes.
// 2. set signal handlers to default (/)
// 3. Set stdio/stdin of processes (redirects) (/)
// 4. Execvp the process (/)
fn launch_process(cmd: &Command, pgid: Pid, foreground: bool) -> Result<(), nix::Error> {
    if stdin().is_terminal() {
        let pid = getpid();
        if pgid.as_raw() == 0 {
            setpgid(pid, pid).unwrap();
        } else {
            setpgid(pid, pgid).unwrap();
        }

        if foreground {
            tcsetpgrp(stdin(), pgid).unwrap();
        }
        default_signal_handlers();
    }

    apply_redirect(&cmd.redirect)?;

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

// Foreach process in pipeline
// 1. Set up pipes ()
// 2. Fork the child process:
//    - Child: launch_process ()
//    - Parent: set pgid of the child process to ... TODO: Think about this
// 3. Cleanup after your pipes TODO: make sure
//
// Then wait for jobs to complete or put in foreground or background
// TODO: Are we doing right ownership of FDs?
fn launch_job(stmt: &Statement) -> Result<(), nix::Error> {
    let mut prev_read: Option<OwnedFd> = None;
    let mut children = Vec::new();
    let pgid = getpgid(None).unwrap();

    for (i, cmd) in stmt.pipeline.commands.iter().enumerate() {
        // setup pipes
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
                launch_process(cmd, pgid, !stmt.background);
            }
            ForkResult::Parent { child, .. } => {
                // TODO: set proper pgid
                children.push(child);

                // cleanup after pipes
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

    if !stdin().is_terminal() {
        for pid in children {
            waitpid(pid, None);
        }
    } else if stmt.background {
        do_bg();
    } else {
        do_fg();
    }
    Ok(())
}

pub fn exec(ast: &Program) -> u64 {
    let statements = &ast.statements;
    statements.iter().for_each(|stmt| {
        exec_statement(stmt);
    });
    return 0;
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
        // setup pipes
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
