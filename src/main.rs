use std::{env, process};

use renamer::Config;

fn main() {
    let args = renamer::handle_special_commands(env::args()).unwrap_or_else(|| process::exit(0));

    let config = Config::build(args.into_iter()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    if let Err(e) = renamer::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    };
}
