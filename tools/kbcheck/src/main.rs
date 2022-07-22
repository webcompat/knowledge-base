use clap::{Parser, Subcommand};
use kbcheck::{data, updates, validate};
use miette::{Diagnostic, Result};
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
    Updates,
}

fn get_tags(root_path: &Path) -> Result<BTreeMap<String, (String, u64)>> {
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

fn tags(root_path: &Path) -> Result<()> {
    let tags = get_tags(root_path)?;
    let mut all_tags = tags.values().collect::<Vec<_>>();
    all_tags.sort();
    for (name, count) in all_tags.iter() {
        println!("{}:{}", name, count);
    }
    Ok(())
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ValidateError {
    #[error("Validation failed")]
    ValidationFailed,
    #[error(transparent)]
    UnexpectedError(#[from] validate::ValidateError),
}

fn validate(root_path: &Path) -> Result<()> {
    let errors = validate::validate(root_path)?;
    if !errors.is_empty() {
        for err in errors {
            println!("{}", err);
        }
        Err(ValidateError::ValidationFailed.into())
    } else {
        Ok(())
    }
}

fn updates(root_path: &Path) -> Result<()> {
    let updates = updates::check_updates(root_path)?;
    if !updates.is_empty() {
        for (path, updates) in updates.iter() {
            println!("{}", path.display());
            for update in updates.iter() {
                println!("    {}\n      Try: {}", update.error, update.suggestion);
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root_path = cli.root_path.unwrap_or_default();
    match &cli.command {
        Commands::Tags => tags(&root_path),
        Commands::Validate => validate(&root_path),
        Commands::Updates => updates(&root_path),
    }?;
    Ok(())
}
