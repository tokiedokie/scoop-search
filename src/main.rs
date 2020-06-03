use std::env;
use std::process;

use scoop_search::scoop::Scoop;
use scoop_search::{parse_args, run};

fn main() {
    let scoop = Scoop::new();

    let args = parse_args(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    if let Err(e) = run(&scoop, &args) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
