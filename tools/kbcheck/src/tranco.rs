use chrono::{DateTime, Duration, NaiveDate, Utc};
use miette::Diagnostic;
use reqwest;
use std::collections::BTreeMap;
use std::fs::create_dir_all;
use std::fs::{read_dir, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::u64;
use thiserror;
use zip::{self, ZipArchive};

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum TrancoError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
    #[error("Tranco Parse Error")]
    TrancoParseError(String),
}

#[derive(Debug, Default)]
struct DataNode {
    children: BTreeMap<String, DataNode>,
    ranking: u64,
}

impl<'a> DataNode {
    fn insert(&mut self, domain_parts: &str, ranking: u64) -> Result<(), TrancoError> {
        if let Some((rest, key)) = domain_parts.rsplit_once('.') {
            self.children
                .entry(key.into())
                .or_insert_with(DataNode::default)
                .insert(rest, ranking)?;
        } else {
            let terminal = self
                .children
                .entry(domain_parts.into())
                .or_insert_with(DataNode::default);
            if terminal.ranking != 0 {
                return Err(TrancoError::TrancoParseError("Got duplicate domain".into()));
            }
            terminal.ranking = ranking;
        }
        Ok(())
    }

    fn get_ranking(&self, matched: &mut Vec<&'a str>, domain_parts: &'a str) -> Option<u64> {
        if let Some((rest, key)) = domain_parts.rsplit_once('.') {
            self.children.get(key).and_then(|child| {
                matched.push(key);
                child.get_ranking(matched, rest).or(if child.ranking == 0 {
                    None
                } else {
                    Some(child.ranking)
                })
            })
        } else {
            self.children.get(domain_parts).and_then(|child| {
                matched.push(domain_parts);
                if child.ranking == 0 {
                    None
                } else {
                    Some(child.ranking)
                }
            })
        }
    }
}

#[derive(Debug, Default)]
pub struct TrancoData {
    root: DataNode,
}

impl<'a> TrancoData {
    pub fn new() -> Self {
        TrancoData {
            root: DataNode::default(),
        }
    }

    pub fn insert(&mut self, domain: &str, ranking: u64) -> Result<(), TrancoError> {
        self.root.insert(domain, ranking).map_err(|err| match err {
            TrancoError::TrancoParseError(msg) => {
                TrancoError::TrancoParseError(format!("{} inserting {}", msg, domain))
            }
            _ => err,
        })
    }

    pub fn get_ranking(&self, domain: &'a str) -> Option<(Vec<&'a str>, u64)> {
        let mut matched = Vec::new();
        self.root
            .get_ranking(&mut matched, domain)
            .map(|ranking| (matched, ranking))
    }
}

fn read_data(stream: impl Read) -> Result<TrancoData, TrancoError> {
    let mut data = TrancoData::new();
    let reader = BufReader::new(stream);
    for read_line in reader.lines() {
        let line = read_line?;
        let parts = line.splitn(2, ',').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(TrancoError::TrancoParseError(format!(
                "Expected comma-separated string, got {}",
                line
            )));
        }
        let ranking_str = parts[0];
        let domain = parts[1];
        let ranking: u64 = ranking_str.parse().map_err(|_| {
            TrancoError::TrancoParseError(format!("Expected integer, got {}", ranking_str))
        })?;
        data.insert(domain, ranking)?;
    }
    Ok(data)
}

pub fn read_tranco_data(path: &Path) -> Result<TrancoData, TrancoError> {
    let file = File::open(path)?;
    if path
        .extension()
        .map(|ext| ext.to_string_lossy())
        .unwrap_or_else(|| "".into())
        == "zip"
    {
        let mut archive = ZipArchive::new(file)?;
        if archive.len() != 1 {
            return Err(TrancoError::TrancoParseError(
                "Got multiple entries in zip archive".into(),
            ));
        }
        let file = archive.by_index(0)?;
        read_data(file)
    } else {
        read_data(file)
    }
}

pub fn download(path: &Path) -> Result<PathBuf, TrancoError> {
    println!("Downloading new tranco data");
    let mut resp =
        reqwest::blocking::get("https://tranco-list.eu/top-1m.csv.zip")?.error_for_status()?;
    let utc_date_time: DateTime<Utc> = Utc::now();
    let dest_path = path
        .to_owned()
        .join(format!("{}.zip", utc_date_time.format("%Y-%m-%d")));
    let file = File::create(&dest_path)?;
    let mut writer = BufWriter::new(file);
    resp.copy_to(&mut writer)?;
    Ok(dest_path)
}

pub fn get_data_file(dir_path: &Path, max_age: usize) -> Result<PathBuf, TrancoError> {
    let paths = read_dir(dir_path)?;
    let today = Utc::now().date().naive_utc();
    let first_date = today - Duration::days(max_age as i64);
    let mut found_file = None;
    for path_entry in paths {
        let path = path_entry?.path();
        let opt_name = path.file_stem();
        if opt_name.is_none() {
            continue;
        }
        let name = opt_name.expect("Name should not be None").to_string_lossy();
        let parsed_date = NaiveDate::parse_from_str(&name, "%Y-%m-%d");
        if parsed_date.is_err() {
            continue;
        }
        let date = parsed_date.expect("Date should be Ok");
        if date >= first_date {
            match found_file {
                None => found_file = Some((date, path)),
                Some((prev_date, _)) if (date > prev_date) => found_file = Some((date, path)),

                _ => {}
            }
        }
    }
    let use_path = found_file
        .map(|(_, path)| Ok(path))
        .unwrap_or_else(|| download(dir_path))?;
    Ok(use_path)
}

pub fn load_recent(dir_path: &Path, max_age: usize) -> Result<(PathBuf, TrancoData), TrancoError> {
    if !dir_path.exists() {
        create_dir_all(&dir_path)?;
    }
    let data_path = get_data_file(dir_path, max_age)?;
    let tranco_data = read_tranco_data(&data_path)?;
    Ok((data_path, tranco_data))
}
