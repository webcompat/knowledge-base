use jsonschema::JSONSchema;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn read_json(path: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

fn read_yaml_as_json(path: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let value: serde_json::Value = serde_yaml::from_reader(reader)?;
    Ok(value)
}

fn validate(root_path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut errors: Vec<String> = Vec::new();
    let schema_path = root_path
        .to_owned()
        .join(PathBuf::from("schemas/entry.schema.json"));
    let data_path = root_path.to_owned().join("data");
    let schema_json = read_json(&schema_path)?;
    let schema = JSONSchema::compile(&schema_json).unwrap();
    for maybe_entry in WalkDir::new(&data_path) {
        let entry = maybe_entry?;
        let path = entry.path();
        let ext = path.extension().unwrap_or_default();
        if ext == "yml" || ext == "yaml" {
            println!("Validating {}", path.display());
            match read_yaml_as_json(path) {
                Ok(json_data) => {
                    let result = schema.validate(&json_data);
                    if let Err(file_errors) = result {
                        for error in file_errors {
                            errors.push(format!("{}: {}", path.display(), error));
                        }
                    };
                }
                Err(err) => {
                    errors.push(err.to_string());
                }
            }
        }
    }
    Ok(errors)
}

enum CommandStatus {
    Ok,
    ValidationError,
    CommandError,
}

fn run() -> CommandStatus {
    let root_path: PathBuf = ["..", ".."].iter().collect();
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
