use clap::{Parser, Subcommand};
use kbcheck::{data, score, tranco, validate};
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
    Score {
        #[clap(last = true, value_parser)]
        data_paths: Vec<PathBuf>,
    },
    Tranco {
        #[clap(long, value_parser)]
        cache_path: Option<PathBuf>,
        domain: String,
        #[clap(long, value_parser, default_value_t = 7)]
        max_age: usize,
    },
}

fn get_tags(root_path: &Path) -> Result<BTreeMap<String, (String, u64)>> {
    let mut tags = BTreeMap::new();
    for entry in data::load_all(root_path, true)?.values() {
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

fn scores(root_path: &Path, data_paths: &[&Path]) -> Result<()> {
    let dir_path = default_tranco_dir(root_path);
    let (_, tranco_data) = tranco::load_recent(&dir_path, 7)?;
    let entries = score::entries_by_score(root_path, data_paths, &tranco_data)?;
    for (_, entry, score) in entries {
        println!("{:.2}\t{}", score, entry.title);
    }
    Ok(())
}

fn default_tranco_dir(root_path: &Path) -> PathBuf {
    let mut path = root_path.to_path_buf();
    path.push(".cache");
    path.push("tranco");
    path
}

fn tranco(
    root_path: &Path,
    cache_path: Option<PathBuf>,
    max_age: usize,
    domain: &str,
) -> Result<()> {
    let dir_path = cache_path.unwrap_or_else(|| default_tranco_dir(root_path));
    let (tranco_path, tranco_data) = tranco::load_recent(&dir_path, max_age)?;
    println!("Using tranco data from {}", tranco_path.display());
    let (matched_domain, ranking) = tranco_data
        .get_ranking(domain)
        .map(|(matched, ranking)| {
            let matched_domain = matched.into_iter().rev().collect::<Vec<&str>>().join(".");
            (matched_domain, ranking.to_string())
        })
        .unwrap_or_else(|| (domain.into(), "None".into()));
    println!("{} ranking {}", matched_domain, ranking);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root_path = cli.root_path.unwrap_or_default();
    match &cli.command {
        Commands::Tags => tags(&root_path),
        Commands::Validate => validate(&root_path),
        Commands::Score { data_paths } => scores(
            &root_path,
            &data_paths
                .iter()
                .map(|x| x.as_ref())
                .collect::<Vec<&Path>>(),
        ),
        Commands::Tranco {
            cache_path,
            domain,
            max_age,
        } => tranco(
            &root_path,
            cache_path.as_ref().map(|x| x.to_owned()),
            *max_age,
            domain,
        ),
    }?;
    Ok(())
}
