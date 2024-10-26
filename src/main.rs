use std::{
    fs::File,
    io::{self, ErrorKind, Read},
    path::Path,
    process::ExitCode,
};

use clap::{Parser as _, Subcommand};
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
mod parser;
mod scope;
mod vm;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Run { file: String },
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
            let path = Path::new(&file);
            if !path.exists() {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    format!("{} does not exist", file),
                )
                .into());
            }

            let mut file = File::open(path)?;
            let mut buffer = String::new();

            file.read_to_string(&mut buffer)?;
            let mut lexer = Lexer::new(&buffer);
            let mut parser = Parser::new(&mut lexer, &buffer);
            let compiler = Compiler::new(&mut parser);

            let program = compiler.compile()?;
            tracing::info!("{:#?}", program);

            let vm = VM::new(program);

            vm.run()?;

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
