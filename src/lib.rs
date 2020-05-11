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

    if buckets
        .iter()
        .find(|bucket| bucket.apps.len() > 0)
        .is_some()
    {
        display_apps(&buckets);
    } else if search_remote_buckets(scoop, &buckets, query)?
        .iter()
        .find(|bucket| bucket.apps.len() > 0)
        .is_some()
    {
        let remote_buckets = search_remote_buckets(scoop, &buckets, query)?;
        display_remote_apps(&remote_buckets);
    } else {
        println!("No matches found.");
    }

    Ok(())
}

fn display_apps(buckets: &Vec<Bucket>) {
    for bucket in buckets {
        if bucket.apps.len() > 0 {
            println!("'{}' bucket: ", bucket.name,);
            for app in &bucket.apps {
                if app.version != "" {
                    println!("    {} ({})", app.name, app.version);
                } else {
                    println!("    {}", app.name);
                }
            }
            println!("");
        }
    }
}

fn display_remote_apps(buckets: &Vec<Bucket>) {
    println!("Results from other known buckets...");
    println!("(add them using 'scoop bucket add <name>')");
    println!("");

    display_apps(buckets);
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
        } else {
            result.push(Bucket {
                name: bucket_name.to_string(),
                apps: Vec::new(),
            })
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

fn search_remote_buckets(
    scoop: &Scoop,
    buckets: &Vec<Bucket>,
    query: &str,
) -> Result<Vec<Bucket>, Box<dyn Error>> {
    let mut buckets_file = PathBuf::from(scoop.dir.as_os_str());

    buckets_file.push("apps\\scoop\\current\\buckets.json");
    let buckets_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&buckets_file)?)?;
    let buckets_map = buckets_json.as_object().unwrap();
    // buckets_map.iter().filter(|bucket| bucket.)

    let mut result: Vec<Bucket> = Vec::new();

    for bucket_tuple in buckets_map {
        let bucket = if buckets
            .iter()
            .find(|bucket| &bucket.name == bucket_tuple.0)
            .is_none()
        {
            let mut bucket = PathBuf::from(bucket_tuple.1.as_str().unwrap().to_string());
            let repository = bucket
                .file_stem()
                .unwrap()
                .to_os_string()
                .to_string_lossy()
                .to_string();
            bucket.pop();
            let user = bucket
                .file_stem()
                .unwrap()
                .to_os_string()
                .to_string_lossy()
                .to_string();
            let api_link = format!(
                "https://api.github.com/repos/{}/{}/git/trees/HEAD?recursive=1",
                user, repository
            );

            let apps = search_remote_bucket(&api_link, query)?;

            Bucket {
                name: bucket_tuple.0.to_string(),
                apps,
            }
        } else {
            Bucket {
                name: bucket_tuple.0.to_string(),
                apps: Vec::new(),
            }
        };

        result.push(bucket);
    }

    Ok(result)
}

fn search_remote_bucket(url: &str, query: &str) -> Result<Vec<App>, Box<dyn Error>> {
    let response_json = ureq::get(url).call().into_json()?;
    //println!("{:?}", response_json["tree"][1]["path"]);

    let tree = response_json.get("tree").expect("Can't get repository");

    let filtered: Vec<String> = tree
        .as_array()
        .unwrap()
        .iter()
        .map(|obj| obj["path"].as_str().unwrap().to_string())
        .filter(|path| path.ends_with(".json"))
        .map(|path| path.trim_end_matches(".json").to_string())
        .filter(|path| path.contains(query))
        .collect();

    let apps = filtered
        .iter()
        .map(|name| App {
            name: name.to_string(),
            version: String::new(),
        })
        .collect::<Vec<App>>();

    Ok(apps)
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
