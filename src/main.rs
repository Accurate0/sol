use std::{
    fs::File,
    io::{self, ErrorKind, Read},
    path::Path,
    process::ExitCode,
    str::FromStr,
};

use clap::{Parser as _, Subcommand, ValueEnum};
use compiler::Compiler;
use lexer::Lexer;
use parser::Parser;
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use vm::VM;

mod ast;
mod compiler;
mod instructions;
mod lexer;
mod macros;
mod parser;
mod scope;
mod stdlib;
mod vm;

// TODO: Add support for testing and conditional jumps aka 'if' statements
// TODO: Functions need to return values, by placing in 0th register - this needs to be compiled
// TODO: Add basic type checking - should be done in same pass as parser?
// TODO: Better dump printing
// TODO: Add line numbers to all errors?

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Run {
        file: String,
    },
    Dump {
        file: String,
        #[arg(short, long, default_value_t, value_enum)]
        item: ItemToDump,
    },
}

#[derive(ValueEnum, Clone, Default, Debug)]
enum ItemToDump {
    /// tokens from the lexer
    #[default]
    Tokens,
    /// parsed ast
    Ast,
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

fn main_internal() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Run { file } => {
            let buffer = read_file_to_string(&file)?;

            let lexer = Lexer::new(&buffer);
            let parser = Parser::new(lexer, &buffer);
            let compiler = Compiler::new(parser);

            let program = compiler.compile()?;

            let vm = VM::new(program);

            vm.run()?;
        }
        Commands::Dump { file, item } => {
            let buffer = read_file_to_string(&file)?;

            match item {
                ItemToDump::Tokens => {
                    let tokens = Lexer::new(&buffer).collect::<Vec<_>>();
                    tracing::info!("{:#?}", tokens);
                }
                ItemToDump::Ast => {
                    let lexer = Lexer::new(&buffer);
                    let parser = Parser::new(lexer, &buffer);

                    let ast = parser.flatten().collect::<Vec<_>>();
                    tracing::info!("{:#?}", ast);
                }
                ItemToDump::Bytecode => {
                    let lexer = Lexer::new(&buffer);
                    let parser = Parser::new(lexer, &buffer);
                    let compiler = Compiler::new(parser);

                    let program = compiler.compile()?;
                    tracing::info!("{:#?}", program);
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

    match main_internal() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{}", e);
            ExitCode::FAILURE
        }
    }
}
