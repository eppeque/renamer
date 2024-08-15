use std::{error::Error, fs, io};

use regex::Regex;

const FMT_RX: &str = r"^[^\s{}]*\{(id|s)\}[^\s{}]*$";
const VERSION: &str = "1.0.2";

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

pub fn handle_special_commands(args: impl Iterator<Item = String>) -> Option<Vec<String>> {
    let mut args_vec: Vec<String> = args.collect();
    let first = args_vec.remove(0);

    if args_vec.is_empty() {
        println!("Renamer v{VERSION}");
        println!("Renamer is a CLI tool allowing you to rename a lot of files easily. You can do so in 2 different ways:");
        println!("1. Rename with an id from 0 to the last file in the directory.");
        println!(
            "2. Rename with an input file containing a list of data to inject in the file name."
        );
        println!("Use the 'help' command for more information.");

        return None;
    }

    if args_vec.len() == 1 && &args_vec[0] == "help" {
        println!("Usage:");
        println!("$ renamer <FILENAME_FORMAT> [INPUT_FILE_PATH]");
        println!("1. FILENAME_FORMAT: This is the format used to build file names.");
        println!("To use the id, type {{id}} (Example: my_file_{{id}}.txt).");
        println!("To use input data, type {{s}} (Example: file_owned_by_{{s}}.txt).");
        println!(
            "2. INPUT_FILE_PATH: This argument is only necessary when using {{s}} format type."
        );
        println!("It indicates the path leading to the input file containing a list with a line for each file.");

        return None;
    }

    args_vec.insert(0, first);
    Some(args_vec)
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
    let mut file_names: Vec<String> = fs::read_dir(".")?
        .map(|os_str| os_str.unwrap().file_name().to_str().unwrap().to_string())
        .collect();

    file_names.sort();

    for (i, current_name) in file_names.into_iter().enumerate() {
        let new_name = fmt.replace("{id}", &(i + 1).to_string());
        fs::rename(current_name, new_name)?;
    }

    Ok(())
}

#[derive(Debug)]
enum InputFileRenamingError {
    FileReading,
    DirReading,
    MismatchedCount,
    RenameError(io::Error),
}

impl std::fmt::Display for InputFileRenamingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileReading => write!(f, "Failed to read the input file"),
            Self::DirReading => write!(f, "Failed to read the directory entries"),
            Self::MismatchedCount => write!(f, "The number of lines in the input file is different than the number of files in this directory"),
            Self::RenameError(e) => write!(f, "Failed to rename files: {e}"),
        }
    }
}

impl Error for InputFileRenamingError {}

fn rename_with_input_file(fmt: &str, path: String) -> Result<(), InputFileRenamingError> {
    let Ok(content) = fs::read_to_string(path) else {
        return Err(InputFileRenamingError::FileReading);
    };

    let mut file_names: Vec<String> = match fs::read_dir(".") {
        Ok(value) => value
            .into_iter()
            .map(|os_str| os_str.unwrap().file_name().to_str().unwrap().to_string())
            .collect(),
        Err(_) => return Err(InputFileRenamingError::DirReading),
    };

    file_names.sort();

    if content.lines().count() != file_names.len() {
        return Err(InputFileRenamingError::MismatchedCount);
    }

    for (line, current_name) in content.lines().zip(file_names.into_iter()) {
        let new_name = fmt.replace("{s}", line);

        if let Err(e) = fs::rename(current_name, new_name) {
            return Err(InputFileRenamingError::RenameError(e));
        };
    }

    Ok(())
}
