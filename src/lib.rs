use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Bucket {
    pub name: String,
    pub apps: Vec<App>,
}

pub struct App {
    pub name: String,
    pub version: String
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

    Ok(query.to_lowercase())
}

fn get_scoop_dir() -> PathBuf {
    // Todo: original scoop search $env:SCOOP, (get_config 'rootPath'), "$env:USERPROFILE\scoop"
    // so this is not enough.
    let mut userprofile = PathBuf::from(env::var("USERPROFILE").unwrap());
    userprofile.push("scoop");
    userprofile
}

pub fn get_bucket(scoop: &Scoop, query: &str) -> Result<Vec<Bucket>, Box<dyn Error>> {
    let buckets = fs::read_dir(&scoop.buckets_dir)?;
    let mut result = Vec::new();

    for bucket in buckets {
        let mut bucket = bucket?.path();
        bucket.push("bucket");

        let apps = fs::read_dir(&bucket)?;

        let file_stems: Vec<String> = apps
            .map(|app| app.unwrap().path().file_stem().unwrap().to_string_lossy().to_string())
            .filter(|file_name| file_name.contains(query))
            .collect();

        if file_stems.len() > 0 {
            let mut apps: Vec<App> = Vec::new();

            for file_stem in &file_stems {
                let mut path = bucket.clone();
                path.push(format!("{}.json", &file_stem));
                let version = get_latest_version(&path)?;
                apps.push(App {
                    name: file_stem.to_string(),
                    version,
                })
                //let app = fs::read_to_string(PathBuf::from("file_stem")).unwrap();
            }


            result.push(Bucket {
                name: bucket.to_string_lossy().to_string(),
                apps,
            });
        }
    }

    Ok(result)
}

fn get_latest_version(path: &Path) -> Result<String, Box<dyn Error>> {
    let manufest = fs::read_to_string(&path)?;
    let manufest_json: serde_json::Value = serde_json::from_str(&manufest)?;
    let version: String = manufest_json["version"].as_str().unwrap().to_string();
    println!("{}", version);

    Ok(version)
}
