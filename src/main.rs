use std::env;
use std::fs;
use std::process;
use std::path::PathBuf;

fn main() {
    let scoop = Scoop::new();
    let query = get_query(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    get_bucket(&scoop, &query);
}

struct Scoop {
    dir: PathBuf,
    buckets_dir: PathBuf,
}

impl Scoop {
    fn new() -> Scoop {
        let dir = get_scoop_dir();
        let mut buckets_dir = PathBuf::from(dir.to_str().unwrap());//PathBuf::new();
        buckets_dir.push("buckets");
        Scoop { dir, buckets_dir }
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

fn get_scoop_dir() -> PathBuf {
    // Todo: original scoop search $env:SCOOP, (get_config 'rootPath'), "$env:USERPROFILE\scoop"
    // so this is not enough.
    let mut userprofile = PathBuf::from(env::var("USERPROFILE").unwrap());
    userprofile.push("scoop");
    userprofile
}

fn get_bucket(scoop: &Scoop, query: &str) {
    //println!("{}", scoop.buckets_dir);

    let buckets = fs::read_dir(&scoop.buckets_dir).unwrap();

    for bucket in buckets {
        let bucket = bucket.unwrap();
        let apps = fs::read_dir(bucket.path());
        for app in apps {
            println!("{:?}", app);
        }
    }
}
