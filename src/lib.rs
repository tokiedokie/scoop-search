use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Bucket {
    pub name: String,
    pub apps: Vec<App>,
}
/*
impl Bucket {
    fn new(name, ) -> Bucket {

    }
}
*/
pub struct App {
    pub name: String,
    pub version: String,
}

impl App {
    fn new(path: PathBuf) -> App {
        let name = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        let version = get_latest_version(&path).unwrap();
        App { name, version }
    }
}

pub struct Scoop {
    dir: PathBuf,
    buckets_dir: PathBuf,
}

impl Scoop {
    pub fn new() -> Scoop {
        let dir = get_scoop_dir().unwrap();
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

fn get_scoop_dir() -> Result<PathBuf, Box<dyn Error>> {
    let scoop_dir = if env::var("SCOOP").is_ok() {
        PathBuf::from(env::var("SCOOP")?)
    } else if has_root_path()? {
        let mut user_profile = PathBuf::from(env::var("USERPROFILE")?);
        user_profile.push(".config");
        user_profile.push("scoop");
        user_profile.push("config.json");
        let config_file = fs::read_to_string(&user_profile)?;
        let config: serde_json::Value = serde_json::from_str(&config_file)?;
        PathBuf::from(config["rootDir"].as_str().unwrap().to_string())
    } else {
        let mut user_profile = PathBuf::from(env::var("USERPROFILE")?);
        user_profile.push("scoop");
        user_profile
    };
    Ok(scoop_dir)
}

fn has_root_path() -> Result<bool, Box<dyn Error>> {
    let mut user_profile = PathBuf::from(env::var("USERPROFILE")?);
    user_profile.push(".config");
    user_profile.push("scoop");
    user_profile.push("config.json");
    let config_file = fs::read_to_string(&user_profile)?;
    let config: serde_json::Value = serde_json::from_str(&config_file)?;
    Ok(config.get("rootPath").is_some())
}

pub fn run(scoop: &Scoop, query: &str) -> Result<(), Box<dyn Error>> {
    let buckets = search_local_buckets(scoop, query)?;
    if buckets.len() == 0 {
        println!("No matches found.");
    } else {
        display_apps(&buckets);
    }
    Ok(())
}

fn display_apps(buckets: &Vec<Bucket>) {
    for bucket in buckets {
        println!("'{}' bucket: ", bucket.name,);
        for app in &bucket.apps {
            println!("    {} ({})", app.name, app.version);
        }
        println!("");
    }
}

fn search_local_buckets(scoop: &Scoop, query: &str) -> Result<Vec<Bucket>, Box<dyn Error>> {
    let buckets = fs::read_dir(&scoop.buckets_dir)?;
    let mut result = Vec::new();

    for bucket in buckets {
        let mut bucket = bucket?.path();
        let bucket_name = &bucket.file_name().unwrap().to_string_lossy().to_string();
        bucket.push("bucket");

        let apps = fs::read_dir(&bucket)?;

        let file_stems: Vec<String> = apps
            .map(|app| {
                app.unwrap()
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .filter(|file_name| file_name.contains(query))
            .collect();

        if file_stems.len() > 0 {
            let mut apps: Vec<App> = Vec::new();

            for file_stem in &file_stems {
                let mut path = bucket.clone();
                path.push(format!("{}.json", &file_stem));
                apps.push(App::new(path));
            }

            result.push(Bucket {
                name: bucket_name.to_string(),
                apps,
            });
        }
    }

    Ok(result)
}

//fn search_query_in(bucket: Bucket, query: &str) -> Result<Bucket, Box<dyn Error>> {}

fn get_latest_version(path: &Path) -> Result<String, Box<dyn Error>> {
    let manufest = fs::read_to_string(&path)?;
    let manufest_json: serde_json::Value = serde_json::from_str(&manufest)?;
    let version: String = manufest_json["version"].as_str().unwrap().to_string();

    Ok(version)
}

fn search_remote_buckets() {

}

#[cfg(test)]
mod test {
    use super::*;

    /*
    #[test]
    fn remote_new() {
        let reference = App{
            name: "rust".to_string(),
            version: "1.43.1".to_string(),
        };
        let target = App::remote_new("rust", "https://api.github.com/repos/ScoopInstaller/Main/git/blobs/1fecc0ecd5aa2af76261ca0fc258b535a0843f9f");
        assert_eq!(reference.name, target.name);
        assert_eq!(reference.version, target.version);
    }
    */
}
