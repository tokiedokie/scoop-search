use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Bucket {
    pub name: String,
    pub apps: Vec<String>,
}

pub struct App {
    name: String,
    version: String
}

pub struct Scoop {
    dir: PathBuf,
    buckets_dir: PathBuf,
}

impl Scoop {
    pub fn new() -> Scoop {
        let dir = get_scoop_dir();
        let mut buckets_dir = PathBuf::from(dir.to_str().unwrap()); //PathBuf::new();
        buckets_dir.push("buckets");
        Scoop { dir, buckets_dir }
    }
}

pub fn get_query(mut args: env::Args) -> Result<String, &'static str> {
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

pub fn get_bucket(scoop: &Scoop, query: &str) -> Vec<Bucket> {
    //println!("{}", scoop.buckets_dir);

    let buckets = fs::read_dir(&scoop.buckets_dir).unwrap();
    let mut result = Vec::new();

    for bucket in buckets {
        let mut bucket = bucket.unwrap().path();
        bucket.push("bucket");

        let apps = fs::read_dir(&bucket).unwrap();

        let file_name: Vec<String> = apps
            .map(|app| app.unwrap().path().file_stem().unwrap().to_string_lossy().to_string())
            .filter(|file_name| file_name.contains(query))
            .collect();

        if file_name.len() > 0 {
            result.push(Bucket {
                name: bucket.to_string_lossy().to_string(),
                apps: file_name,
            });
        }
    }

    result
}
