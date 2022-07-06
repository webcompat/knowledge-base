use crate::data::{iter_data_files, read_json, EntriesMap};
use crate::entry::Entry;
use jsonschema::JSONSchema;
use serde::de::{Deserialize, IntoDeserializer};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

pub type Failures = Vec<ValidationFailure>;

#[derive(Error, Debug)]
enum ValidateError {
    #[error("Load failed")]
    DataError(
        #[from]
        #[source]
        crate::data::DataError,
    ),
    #[error("Schema compile failed")]
    SchemaError,
}

impl From<jsonschema::ValidationError<'_>> for ValidateError {
    fn from(_: jsonschema::ValidationError<'_>) -> Self {
        ValidateError::SchemaError
    }
}

/// Representation of an error in a data file that causes it to fail validation
pub struct ValidationFailure {
    path: PathBuf,
    line: Option<usize>,
    message: String,
}

impl ValidationFailure {
    fn new(path: &Path, line: Option<usize>, message: String) -> ValidationFailure {
        ValidationFailure {
            path: path.to_path_buf(),
            line,
            message,
        }
    }
}

impl Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{} - {}", self.path.display(), line, self.message),
            None => write!(f, "{} - {}", self.path.display(), self.message),
        }
    }
}

/// Validate the contents of the string data of a file.
/// This can check for purely textual errors like trailing whitespace
fn validate_file_as_string(path: &Path, data: &str, errors: &mut Failures) {
    for (i, line) in data.lines().enumerate() {
        // Check trailing whitespace
        if line.ends_with(|c: char| c.is_ascii_whitespace()) {
            errors.push(ValidationFailure::new(
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
) -> Result<(Option<Entry>, Failures), ValidateError> {
    println!("Loading {}", path.display());
    let mut errors = Vec::new();

    let maybe_file_string = fs::read_to_string(path);
    if let Err(error) = maybe_file_string {
        errors.push(ValidationFailure::new(path, None, error.to_string()));
        return Ok((None, errors));
    };

    let file_string = maybe_file_string.expect("Should be able to unwrap Ok value");

    validate_file_as_string(path, &file_string, &mut errors);

    let maybe_json_value: Result<serde_json::Value, _> = serde_yaml::from_str(&file_string);
    if let Err(error) = maybe_json_value {
        errors.push(ValidationFailure::new(path, None, error.to_string()));
        return Ok((None, errors));
    };

    let json_value = maybe_json_value.expect("Should be able to unwrap Ok value");

    if let Err(file_errors) = schema.validate(&json_value) {
        for error in file_errors {
            errors.push(ValidationFailure::new(path, None, error.to_string()));
        }
        return Ok((None, errors));
    };

    // Convert the JSON Value into an Entry object and add it to the map
    match Entry::deserialize(json_value.into_deserializer()) {
        Ok(entry) => Ok((Some(entry), errors)),
        Err(error) => {
            errors.push(ValidationFailure::new(path, None, error.to_string()));
            Ok((None, errors))
        }
    }
}

fn load_schema(root_path: &Path) -> Result<JSONSchema, ValidateError> {
    let schema_path = root_path
        .to_owned()
        .join(PathBuf::from("schemas/entry.schema.json"));
    let schema_json = read_json(&schema_path)?;
    JSONSchema::compile(&schema_json).map_err(ValidateError::from)
}

/// Load all the data files, validate each individual file, and return
/// a map between a PathBuf and the Entry object for that file, along
/// with a list of validation errors.
fn load_and_validate_files(root_path: &Path) -> Result<(EntriesMap, Failures), ValidateError> {
    let mut entries: EntriesMap = BTreeMap::new();
    let mut errors = Vec::new();
    let schema = load_schema(root_path)?;

    for maybe_dir_entry in iter_data_files(root_path) {
        let entry = maybe_dir_entry?;
        let path = entry.path();
        let (maybe_entry, file_errors) = load_and_validate_file(&schema, path)?;
        if let Some(entry) = maybe_entry {
            entries.insert(path.to_path_buf(), entry);
        }
        errors.extend(file_errors.into_iter())
    }
    Ok((entries, errors))
}

/// Validate that more than one file doesn't correspond to the same platform issue.
/// This is highly suggestive that the issues should be merged or the bugs split
fn validate_unique_platform_issues(entries: &EntriesMap) -> Failures {
    let mut errors = Vec::new();
    let mut seen_issues: BTreeMap<Url, &Path> = BTreeMap::new();
    for (path, entry) in entries.iter() {
        for issue_url in entry.references.platform_issues.iter() {
            let mut resource_url = issue_url.clone();
            resource_url.set_fragment(None);
            match seen_issues.get(&resource_url) {
                Some(first_path) => {
                    errors.push(ValidationFailure::new(
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
fn global_validate(entries: EntriesMap) -> Failures {
    let mut errors = Vec::new();
    errors.extend(validate_unique_platform_issues(&entries).into_iter());
    errors
}

/// Validate knowledge base entries
pub fn validate(root_path: &Path) -> Result<Failures, Box<dyn std::error::Error>> {
    let mut errors: Failures = Vec::new();
    let (entries, file_errors) = load_and_validate_files(root_path)?;
    errors.extend(file_errors.into_iter());
    errors.extend(global_validate(entries).into_iter());
    Ok(errors)
}
