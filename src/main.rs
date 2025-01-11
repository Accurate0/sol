use clap::{Parser as _, Subcommand, ValueEnum};
use codespan_reporting::{
    files::SimpleFiles,
    term::termcolor::{ColorChoice, StandardStream},
};
use compiler::Compiler;
use lexer::Lexer;
use parser::Parser;
use std::{
    fs::File,
    io::{self, ErrorKind, Read},
    path::Path,
    process::ExitCode,
    str::FromStr,
};
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use typechecker::Typechecker;
use vm::VM;

mod ast;
mod compiler;
mod instructions;
mod lexer;
mod macros;
mod parser;
mod scope;
mod stdlib;
mod typechecker;
mod types;
mod vm;

// TODO: Better errors, like Rust
// TODO: Add arrays that aren't just objects with number indexes
// TODO: Better dump printing
// TODO: Add ability to include other files? C-style #include? files that are included can't have
//       global code, only the "main" file can... for now
// TODO: Add generic statemap type thing passed to each stdlib function
//       This will let me trivially add networking
//       Also move to another crate due to dependencies
// TODO: Increase register count / reuse registers in some way....
//       Find a way to reclaim registers once they are proved unused
//       Collect a list of registers as we parse that won't be reused in a scope
//       And every time we need one, check this list
// TODO: Allow anonymous functions
// TODO: Allows functions in objects in some way
// TODO: Multi-crate setup
// TODO: LSP :)

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// run a program file
    Run {
        file: String,
        #[arg(short, long, default_value_t = false)]
        no_typecheck: bool,
    },
    /// dump internal state
    Dump {
        file: String,
        #[arg(short, long, default_value_t, value_enum)]
        target: DumpTarget,
        #[arg(long, default_value_t = false)]
        typecheck: bool,
    },
}

#[derive(ValueEnum, Clone, Default, Debug)]
enum DumpTarget {
    /// tokens from the lexer
    #[default]
    Tokens,
    /// parsed ast
    Ast,
    /// typechecked ast
    Typecheck,
    /// compiled bytecode
    Bytecode,
}

fn read_file_to_string(path_unchecked: &str) -> Result<String, std::io::Error> {
    let path = Path::new(&path_unchecked);
    if !path.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("{} does not exist", path_unchecked),
        ));
    }

    let mut file = File::open(path)?;
    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;

    Ok(buffer)
}

fn main_internal(no_color: bool) -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut code_reporting_file_db = SimpleFiles::new();
    let color = if no_color {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };

    let writer = StandardStream::stderr(color);
    let config = codespan_reporting::term::Config::default();

    match args.command {
        Commands::Run { file, no_typecheck } => {
            let buffer = read_file_to_string(&file)?;
            let file_id = code_reporting_file_db.add(&file, &buffer);

            let lexer = Lexer::new(file_id, &buffer);
            let parser = Parser::new(lexer, &buffer);

            let statements = parser
                .collect_and_emit_diagnostics(writer, config, &code_reporting_file_db)
                .ok_or("")?;

            if !no_typecheck {
                let typechecker = Typechecker::default();
                typechecker.check(&statements)?;
            }

            let compiler = Compiler::new();
            let program = compiler.compile(&statements)?;

            let vm = VM::new(program);

            vm.run()?;
        }
        Commands::Dump {
            file,
            target,
            typecheck,
        } => {
            let buffer = read_file_to_string(&file)?;

            match target {
                DumpTarget::Tokens => {
                    let tokens = Lexer::new(0, &buffer).collect::<Vec<_>>();
                    tracing::info!("{:#?}", tokens);
                }
                DumpTarget::Ast => {
                    let lexer = Lexer::new(0, &buffer);
                    let parser = Parser::new(lexer, &buffer);

                    let statements = parser
                        .collect_and_emit_diagnostics(writer, config, &code_reporting_file_db)
                        .ok_or("")?;

                    if typecheck {
                        let typechecker = Typechecker::default();
                        typechecker.check(&statements)?;
                    }

                    tracing::info!("{statements:#?}")
                }
                DumpTarget::Bytecode => {
                    let lexer = Lexer::new(0, &buffer);
                    let parser = Parser::new(lexer, &buffer);

                    let statements = parser
                        .collect_and_emit_diagnostics(writer, config, &code_reporting_file_db)
                        .ok_or("")?;

                    if typecheck {
                        let typechecker = Typechecker::default();
                        typechecker.check(&statements)?;
                    }

                    let compiler = Compiler::new();

                    let program = compiler.compile(&statements)?;
                    tracing::info!("{:#?}", program);
                }
                DumpTarget::Typecheck => {
                    let lexer = Lexer::new(0, &buffer);
                    let parser = Parser::new(lexer, &buffer);
                    let typechecker = Typechecker::default();

                    let statements = parser
                        .collect_and_emit_diagnostics(writer, config, &code_reporting_file_db)
                        .ok_or("")?;

                    typechecker.check(&statements)?;
                }
            }
        }
    };

    Ok(())
}

fn main() -> ExitCode {
    let no_color = std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty());
    let log_level = match std::env::var("PLRS_LOG").ok() {
        Some(l) => Level::from_str(&l).unwrap_or(Level::INFO),
        None => Level::INFO,
    };

    tracing_subscriber::registry()
        .with(Targets::default().with_default(log_level))
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_ansi(!no_color)
                .compact(),
        )
        .init();

    match main_internal(no_color) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{}", e);
            ExitCode::FAILURE
        }
    }
}
