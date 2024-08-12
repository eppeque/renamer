use std::{error::Error, fs, io};

use regex::Regex;

const FMT_RX: &str = r"^[^\s{}]*\{(id|s)\}[^\s{}]*$";

pub struct Config {
    fmt: String,
    list_path: Option<String>,
}

impl Config {
    /// # Panics
    /// This function panics if the regex format is invalid (it's not)
    ///
    /// # Errors
    /// This function returns an error if:
    /// - an argument is missing
    /// - the syntax of the format is invalid
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();

        let Some(input) = args.next() else {
            return Err("Please specify the renaming format");
        };

        let rx = Regex::new(FMT_RX).unwrap();

        if !rx.is_match(&input) {
            return Err("The format syntax is invalid");
        }

        let (_, [input_type]) = rx.captures(&input).unwrap().extract();

        let list_path = match args.next() {
            Some(arg) => Some(arg),
            None if input_type == "s" => {
                return Err("Please provide the path to a list when using the {s} input type")
            }
            None => None,
        };

        Ok(Self {
            fmt: input,
            list_path,
        })
    }
}

/// # Errors
/// This function returns an error if the application code fails at some point.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.list_path {
        Some(path) => rename_with_input_file(&config.fmt, path)?,
        None => rename_with_index(&config.fmt)?,
    };

    Ok(())
}

fn rename_with_index(fmt: &str) -> Result<(), io::Error> {
    let entries = fs::read_dir(".")?;

    for (i, entry) in entries.enumerate() {
        let current_name = entry?.file_name();
        let new_name = fmt.replace("{id}", &i.to_string());

        fs::rename(current_name, new_name)?;
    }

    Ok(())
}

#[derive(Debug)]
enum InputFileRenamingError {
    FileReading,
    DirReading,
    MismatchedCount,
    InsufficientPermissions,
    RenameError(io::Error),
}

impl std::fmt::Display for InputFileRenamingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileReading => write!(f, "Failed to read the input file"),
            Self::DirReading => write!(f, "Failed to read the directory entries"),
            Self::MismatchedCount => write!(f, "The number of lines in the input file is different than the number of files in this directory"),
            Self::InsufficientPermissions => write!(f, "An error occured due to insufficient permissions"),
            Self::RenameError(e) => write!(f, "Failed to rename files: {e}"),
        }
    }
}

impl Error for InputFileRenamingError {}

fn rename_with_input_file(fmt: &str, path: String) -> Result<(), InputFileRenamingError> {
    let Ok(content) = fs::read_to_string(path) else {
        return Err(InputFileRenamingError::FileReading);
    };

    let entries_count = match fs::read_dir(".") {
        Ok(value) => value.count(),
        Err(_) => return Err(InputFileRenamingError::DirReading),
    };

    if content.lines().count() != entries_count {
        return Err(InputFileRenamingError::MismatchedCount);
    }

    let Ok(entries) = fs::read_dir(".") else {
        return Err(InputFileRenamingError::DirReading);
    };

    for (line, entry) in content.lines().zip(entries) {
        let new_name = fmt.replace("{s}", line);
        let current_name = match entry {
            Ok(entry) => entry.file_name(),
            Err(_) => return Err(InputFileRenamingError::InsufficientPermissions),
        };

        if let Err(e) = fs::rename(current_name, new_name) {
            return Err(InputFileRenamingError::RenameError(e));
        };
    }

    Ok(())
}
