use kbcheck::validate;
use std::path::PathBuf;

enum CommandStatus {
    Ok,
    Failure,
    UnexpectedError,
}

fn run() -> CommandStatus {
    let root_path = PathBuf::new();
    match validate::validate(&root_path) {
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

fn main() {
    let exit_code = match run() {
        CommandStatus::Ok => 0,
        CommandStatus::Failure => 1,
        CommandStatus::UnexpectedError => 2,
    };
    std::process::exit(exit_code);
}
