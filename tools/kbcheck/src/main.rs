use clap::{Parser, Subcommand};
use kbcheck::validate;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(short, long, value_parser)]
    root_path: Option<PathBuf>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate data files against schema
    Validate,
}

enum CommandStatus {
    Ok,
    Failure,
    UnexpectedError,
}

fn validate(root_path: &Path) -> CommandStatus {
    match validate::validate(root_path) {
        Ok(errors) => {
            if !errors.is_empty() {
                println!("Validation failed");
                for err in errors {
                    println!("{}", err);
                }
                CommandStatus::Failure
            } else {
                CommandStatus::Ok
            }
        }
        Err(err) => {
            println!("{}", err);
            CommandStatus::UnexpectedError
        }
    }
}

fn run() -> CommandStatus {
    let cli = Cli::parse();
    let root_path = cli.root_path.unwrap_or_default();
    match &cli.command {
        Commands::Validate => validate(&root_path),
    }
}

fn main() {
    let exit_code = match run() {
        CommandStatus::Ok => 0,
        CommandStatus::Failure => 1,
        CommandStatus::UnexpectedError => 2,
    };
    std::process::exit(exit_code);
}
