use std::env;
use std::process;

use scoop_search::*;

fn main() {
    let scoop = Scoop::new();

    let query = get_query(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    
    let buckets = search(&scoop, &query).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    
    if buckets.len() == 0 {
        println!("No matches found.");
    }
    for bucket in buckets {
        println!("'{}' bucket: ", bucket.name,);
        for app in bucket.apps {
            println!("{} ({})", app.name, app.version);
        }
        println!("");
    }
}
