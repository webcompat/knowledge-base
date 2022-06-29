mod entry;

use entry::Entry;
use jsonschema::JSONSchema;
use serde::de::{Deserialize, IntoDeserializer};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use url::Url;
use walkdir::WalkDir;

type EntriesMap = BTreeMap<PathBuf, Entry>;
type Errors = Vec<ValidationError>;

/// Representation of an error in a data file that causes it to fail validation
struct ValidationError {
    path: PathBuf,
    line: Option<usize>,
    message: String,
}

impl ValidationError {
    fn new(path: &Path, line: Option<usize>, message: String) -> ValidationError {
        ValidationError {
            path: path.to_path_buf(),
            line,
            message,
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{} - {}", self.path.display(), line, self.message),
            None => write!(f, "{} - {}", self.path.display(), self.message),
        }
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

/// Validate the contents of the string data of a file.
/// This can check for purely textual errors like trailing whitespace
fn validate_file_as_string(path: &Path, data: &str, errors: &mut Errors) {
    for (i, line) in data.lines().enumerate() {
        // Check trailing whitespace
        if line.ends_with(|c: char| c.is_ascii_whitespace()) {
            errors.push(ValidationError::new(
                path,
                Some(i),
                "trailing whitespace".into(),
            ));
        }
    }
}

/// Load each data file and perform per-file validation For files that
/// could be successfully parsed, this returns an Entry object. In all
/// cases it returns a list of validation errors.  Note that in the
/// case that the file can be parsed despite a validation error, we
/// return the Entry object for further processing.
fn load_and_validate_file(
    schema: &JSONSchema,
    path: &Path,
) -> Result<(Option<Entry>, Errors), Box<dyn Error>> {
    println!("Loading {}", path.display());
    let mut errors = Vec::new();

    let maybe_file_string = fs::read_to_string(path);
    if let Err(error) = maybe_file_string {
        errors.push(ValidationError::new(path, None, error.to_string()));
        return Ok((None, errors));
    };

    let file_string = maybe_file_string.expect("Should be able to unwrap Ok value");

    validate_file_as_string(path, &file_string, &mut errors);

    let maybe_json_value: Result<serde_json::Value, _> = serde_yaml::from_str(&file_string);
    if let Err(error) = maybe_json_value {
        errors.push(ValidationError::new(path, None, error.to_string()));
        return Ok((None, errors));
    };

    let json_value = maybe_json_value.expect("Should be able to unwrap Ok value");

    if let Err(file_errors) = schema.validate(&json_value) {
        for error in file_errors {
            errors.push(ValidationError::new(path, None, error.to_string()));
        }
        return Ok((None, errors));
    };

    // Convert the JSON Value into an Entry object and add it to the map
    match Entry::deserialize(json_value.into_deserializer()) {
        Ok(entry) => Ok((Some(entry), errors)),
        Err(error) => {
            errors.push(ValidationError::new(path, None, error.to_string()));
            Ok((None, errors))
        }
    }
}

/// Load all the data files, validate each individual file, and return
/// a map between a PathBuf and the Entry object for that file, along
/// with a list of validation errors.
fn load_and_validate_files(root_path: &Path) -> Result<(EntriesMap, Errors), Box<dyn Error>> {
    let schema_path = root_path
        .to_owned()
        .join(PathBuf::from("schemas/entry.schema.json"));
    let data_path = root_path.to_owned().join("data");
    let schema_json = read_json(&schema_path)?;
    let schema = JSONSchema::compile(&schema_json).unwrap();

    let mut entries: EntriesMap = BTreeMap::new();
    let mut errors = Vec::new();

    for maybe_entry in WalkDir::new(&data_path) {
        let entry = maybe_entry?;
        let path = entry.path();
        let ext = path.extension().unwrap_or_default();
        if ext == "yml" || ext == "yaml" {
            let (maybe_entry, file_errors) = load_and_validate_file(&schema, path)?;
            if let Some(entry) = maybe_entry {
                entries.insert(path.to_path_buf(), entry);
            }
            errors.extend(file_errors.into_iter())
        }
    }
    Ok((entries, errors))
}

/// Validate that more than one file doesn't correspond to the same platform issue.
/// This is highly suggestive that the issues should be merged or the bugs split
fn validate_unique_platform_issues(entries: &EntriesMap) -> Errors {
    let mut errors = Vec::new();
    let mut seen_issues: BTreeMap<Url, &Path> = BTreeMap::new();
    for (path, entry) in entries.iter() {
        for issue_url in entry.references.platform_issues.iter() {
            let mut resource_url = issue_url.clone();
            resource_url.set_fragment(None);
            match seen_issues.get(&resource_url) {
                Some(first_path) => {
                    errors.push(ValidationError::new(
                        path,
                        None,
                        format!(
                            "Duplicate issue URL {} first seen in {}",
                            resource_url,
                            first_path.display()
                        ),
                    ));
                }
                None => {
                    seen_issues.insert(resource_url, path);
                }
            }
        }
    }
    errors
}

/// Validate properties of the knowledge base as a whole
fn global_validate(entries: EntriesMap) -> Errors {
    let mut errors = Vec::new();
    errors.extend(validate_unique_platform_issues(&entries).into_iter());
    errors
}

/// Validate knowledge base entries
fn validate(root_path: &Path) -> Result<Errors, Box<dyn Error>> {
    let mut errors: Errors = Vec::new();
    let (entries, file_errors) = load_and_validate_files(root_path)?;
    errors.extend(file_errors.into_iter());
    errors.extend(global_validate(entries).into_iter());
    Ok(errors)
}

enum CommandStatus {
    Ok,
    ValidationError,
    CommandError,
}

fn run() -> CommandStatus {
    let root_path = PathBuf::new();
    match validate(&root_path) {
        Ok(errors) => {
            if !errors.is_empty() {
                println!("Validation failed");
                for err in errors {
                    println!("{}", err);
                }
                CommandStatus::ValidationError
            } else {
                CommandStatus::Ok
            }
        }
        Err(err) => {
            println!("{}", err);
            CommandStatus::CommandError
        }
    }
}

fn main() {
    let exit_code = match run() {
        CommandStatus::Ok => 0,
        CommandStatus::ValidationError => 1,
        CommandStatus::CommandError => 2,
    };
    std::process::exit(exit_code);
}
