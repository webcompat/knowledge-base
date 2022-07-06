use clap::{Parser, Subcommand};
use kbcheck::{data, validate};
use std::collections::BTreeMap;
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
    /// Print all tags currently used
    Tags,
    /// Validate data files against schema
    Validate,
}

enum CommandStatus {
    Ok,
    Failure,
    UnexpectedError,
}

fn get_tags(root_path: &Path) -> Result<BTreeMap<String, (String, u64)>, data::DataError> {
    let mut tags = BTreeMap::new();
    for entry in data::load_all(root_path)?.values() {
        for tag in entry.tags.iter() {
            let canonical_tag = tag.to_lowercase();
            if !tags.contains_key(&canonical_tag) {
                tags.insert(canonical_tag.clone(), (tag.clone(), 0));
            }
            tags.get_mut(&canonical_tag).expect("Tag should exist").1 += 1;
        }
    }
    Ok(tags)
}

fn tags(root_path: &Path) -> CommandStatus {
    match get_tags(root_path) {
        Ok(tags) => {
            let mut all_tags = tags.values().collect::<Vec<_>>();
            all_tags.sort();
            for (name, count) in all_tags.iter() {
                println!("{}:{}", name, count);
            }
            CommandStatus::Ok
        }
        Err(err) => {
            println!("{}", err);
            CommandStatus::UnexpectedError
        }
    }
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
        Commands::Tags => tags(&root_path),
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
