use std::env;
use std::process;

use scoop_search::*;

fn main() {
    let scoop = Scoop::new();

    let query = get_query(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    /*
    let args = parse_args(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    */

    if let Err(e) = run(&scoop, &query) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
