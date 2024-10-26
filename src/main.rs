use std::{
    fs::File,
    io::{self, ErrorKind, Read},
    path::Path,
    process::ExitCode,
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
mod vm;

// TODO: Add basic type checking - should be done in same pass as parser?

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

fn main() -> ExitCode {
    let no_color = std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty());
    tracing_subscriber::registry()
        .with(Targets::default().with_default(Level::INFO))
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_ansi(!no_color)
                .compact(),
        )
        .init();

    let args = Args::parse();

    let main_internal = || match args.command {
        Commands::Run { file } => {
            let buffer = read_file_to_string(&file)?;

            let mut lexer = Lexer::new(&buffer);
            let mut parser = Parser::new(&mut lexer, &buffer);
            let compiler = Compiler::new(&mut parser);

            let program = compiler.compile()?;
            tracing::info!("{:#?}", program);

            let vm = VM::new(program);

            vm.run()?;

            Ok::<(), Box<dyn std::error::Error>>(())
        }
        Commands::Dump { file, item } => {
            let buffer = read_file_to_string(&file)?;

            match item {
                ItemToDump::Tokens => {
                    let tokens = Lexer::new(&buffer).collect::<Vec<_>>();
                    tracing::info!("{:#?}", tokens);
                }
                ItemToDump::Ast => todo!(),
            }

            Ok::<(), Box<dyn std::error::Error>>(())
        }
    };

    match main_internal() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{}", e);
            ExitCode::FAILURE
        }
    }
}
