use std::env;
use std::fs;
use std::process;

fn main() {
    let scoop = Scoop::new();
    let query = get_query(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    get_bucket(&scoop.dir, query);
}

struct Scoop {
    dir: String,
}

impl Scoop {
    fn new() -> Scoop {
        let dir = get_scoop_dir();
        Scoop { dir }
    }
}

fn get_query(mut args: env::Args) -> Result<String, &'static str> {
    args.next();
    let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query"),
    };

    Ok(query)
}

fn get_scoop_dir() -> String {
    // Todo: original scoop search $env:SCOOP, (get_config 'rootPath'), "$env:USERPROFILE\scoop"
    // so this is not enough.
    let userprofile = env::var("USERPROFILE").unwrap();
    format!("{}\\scoop", userprofile)
}

fn get_bucket(scoop_dir: &str, query: String) {
    let buckets_dir = format!("{}\\buckets", scoop_dir);
    //println!("{}", buckets_dir);

    let buckets = fs::read_dir(buckets_dir).unwrap();
    for path in buckets {
        println!("{:?}", path.unwrap().path().display())
    }
}
