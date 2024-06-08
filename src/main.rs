use std::{
    error::Error,
    fs::File,
    io::{self, ErrorKind, Read},
    path::Path,
};

use clap::{Parser as _, Subcommand};
use compiler::Compiler;
use lexer::Lexer;
use parser::Parser;
use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

mod ast;
mod compiler;
mod instructions;
mod lexer;
mod parser;

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

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(Targets::default().with_default(Level::INFO))
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_ansi(true)
                .compact(),
        )
        .init();

    let args = Args::parse();
    match args.command {
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

            let program = parser.parse()?;
            tracing::info!("{:#?}", program);

            let compiler = Compiler::new(&program);
            let program = compiler.compile()?;

            tracing::info!("{:#?}", program);
        }
    }

    Ok(())
}
