use std::{env, process};

use renamer::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    if let Err(e) = renamer::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    };
}
