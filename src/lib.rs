use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
struct Bucket {
    name: String,
    apps: Vec<App>,
}

impl Bucket {
    fn new(name: String, apps: Vec<App>) -> Bucket {
        Bucket { name, apps }
    }

    fn get_buckets_path(scoop: &Scoop) -> Vec(PathBuf) {
        
    }
}

#[derive(Debug, PartialEq)]
struct App {
    name: String,
    version: String,
    bin: Vec<String>,
}

impl App {
    fn new(path: PathBuf) -> App {
        let name = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        let (version, bin) = App::get_version_bin(&path).unwrap();
        App { name, version, bin }
    }

    fn get_version_bin(path: &Path) -> Result<(String, Vec<String>), Box<dyn Error>> {
        let manufest = fs::read_to_string(&path)?;
        let manufest_json: serde_json::Value = serde_json::from_str(&manufest)?;

        let version: String = manufest_json["version"].as_str().unwrap().to_string();

        let bin: Vec<String> = match manufest_json.get("bin") {
            Some(x) => match x.as_str() {
                Some(bin) => vec![bin.to_string()],
                None => match x.as_array() {
                    Some(bins) => bins
                        .clone()
                        .iter()
                        .map(|bin| match bin.as_str() {
                            Some(str) => str.to_string(),
                            None => String::new(),
                        })
                        .collect(),
                    None => Vec::new(),
                },
            },
            None => Vec::new(),
        };

        Ok((version, bin))
    }
}

#[derive(Debug, PartialEq)]
pub struct Scoop {
    dir: PathBuf,
    buckets_dir: PathBuf,
}

impl Scoop {
    pub fn new() -> Scoop {
        let dir = Scoop::get_scoop_dir().unwrap();
        let mut buckets_dir = PathBuf::from(dir.to_str().unwrap()); //PathBuf::new();
        buckets_dir.push("buckets");
        Scoop { dir, buckets_dir }
    }

    fn get_scoop_dir() -> Result<PathBuf, Box<dyn Error>> {
        let scoop_dir = if env::var("SCOOP").is_ok() {
            PathBuf::from(env::var("SCOOP")?)
        } else if Scoop::has_root_path()? {
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
}

pub fn get_query(mut args: env::Args) -> Result<String, &'static str> {
    args.next();
    let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query"),
    };

    Ok(query.to_lowercase())
}

pub fn run(scoop: &Scoop, query: &str) -> Result<(), Box<dyn Error>> {

    let buckets = get_buckets(scoop).unwrap();
    let filtered_buckets = search_apps(&buckets, query);

    match filtered_buckets {
        Some(buckets) => display_apps(&buckets),
        None => match search_remote_buckets(&scoop, &buckets, query) {
            Some(remote_buckets) => display_remote_apps(&remote_buckets),
            None => println!("No matches Found"),
        },
    }

    Ok(())
}

fn display_apps(buckets: &Vec<Bucket>) {
    for bucket in buckets {
        if bucket.apps.len() > 0 {
            println!("'{}' bucket: ", bucket.name,);
            for app in &bucket.apps {
                if app.version != "" {
                    if app.bin.len() > 0 {
                        println!(
                            "    {} ({}) --> includes '{}'",
                            app.name, app.version, app.bin[0]
                        );
                    } else {
                        println!("    {} ({})", app.name, app.version);
                    }
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

fn search_local_buckets(scoop: &Scoop, query: &str) -> Option<Vec<Bucket>> {

}

fn get_buckets(scoop: &Scoop) -> Result<Vec<Bucket>, Box<dyn Error>> {
    let buckets = fs::read_dir(&scoop.buckets_dir)?;
    let mut result = Vec::new();

    for bucket in buckets {
        let mut bucket = bucket?.path();
        let bucket_name = &bucket.file_name().unwrap().to_string_lossy().to_string();
        bucket.push("bucket");

        let app_files = fs::read_dir(&bucket)?;

        let app_paths: Vec<PathBuf> = app_files.map(|app| app.unwrap().path()).collect();

        let mut apps = Vec::new();

        for app_path in app_paths {
            apps.push(App::new(app_path));
        }

        result.push(Bucket::new(bucket_name.to_string(), apps));
    }

    Ok(result)
}

fn search_apps(buckets: &Vec<Bucket>, query: &str) -> Option<Vec<Bucket>> {
    let mut result: Vec<Bucket> = Vec::new();
    let mut none_flag = true;

    for bucket in buckets {
        let mut filtered_apps: Vec<App> = Vec::new();

        for app in &bucket.apps {
            if app.name.contains(query) {
                filtered_apps.push(App {
                    name: app.name.clone(),
                    version: app.version.clone(),
                    bin: Vec::new(),
                })
            } else {
                for bin in &app.bin {
                    let bin = Path::new(&bin)
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new(""))
                        .to_string_lossy()
                        .to_string();
                    if bin.contains(query) {
                        filtered_apps.push(App {
                            name: app.name.clone(),
                            version: app.version.clone(),
                            bin: vec![bin],
                        })
                    }
                }
            }
        }

        if filtered_apps.len() > 0 {
            none_flag = false;
        }

        result.push(Bucket {
            name: bucket.name.clone(),
            apps: filtered_apps,
        });
    }

    if none_flag {
        return None;
    }

    Some(result)
}

fn search_remote_buckets(scoop: &Scoop, buckets: &Vec<Bucket>, query: &str) -> Option<Vec<Bucket>> {
    let mut buckets_file = PathBuf::from(scoop.dir.as_os_str());

    buckets_file.push("apps\\scoop\\current\\buckets.json");
    let buckets_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&buckets_file).unwrap()).unwrap();
    let buckets_map = buckets_json.as_object()?;

    let mut result: Vec<Bucket> = Vec::new();

    let mut none_flag = true;

    for bucket_tuple in buckets_map {
        let bucket = if buckets
            .iter()
            .find(|bucket| &bucket.name == bucket_tuple.0)
            .is_none()
        {
            let mut bucket = PathBuf::from(bucket_tuple.1.as_str()?.to_string());
            let repository = bucket
                .file_stem()?
                .to_os_string()
                .to_string_lossy()
                .to_string();
            bucket.pop();
            let user = bucket
                .file_stem()?
                .to_os_string()
                .to_string_lossy()
                .to_string();
            let api_link = format!(
                "https://api.github.com/repos/{}/{}/git/trees/HEAD?recursive=1",
                user, repository
            );

            let apps = search_remote_bucket(&api_link, query).unwrap();

            if apps.len() > 0 {
                none_flag = false;
            }

            Bucket::new(bucket_tuple.0.to_string(), apps)
        } else {
            Bucket::new(bucket_tuple.0.to_string(), Vec::new())
        };

        result.push(bucket);
    }

    if none_flag {
        return None;
    }

    Some(result)
}

fn search_remote_bucket(url: &str, query: &str) -> Result<Vec<App>, Box<dyn Error>> {
    let response_json = ureq::get(url).call().into_json()?;

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
            bin: Vec::new(),
        })
        .collect::<Vec<App>>();

    Ok(apps)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search_apps() {
        let buckets = vec![Bucket {
            name: String::from("test_bucket"),
            apps: vec![App {
                name: String::from("test_app"),
                version: String::from("test_version"),
                bin: vec![String::from("test_bin")],
            }],
        }];
        let query = String::from("test");

        let expect = vec![Bucket {
            name: String::from("test_bucket"),
            apps: vec![App {
                name: String::from("test_app"),
                version: String::from("test_version"),
                bin: Vec::new(),
            }],
        }];

        let actual = search_apps(&buckets, &query).unwrap();

        assert_eq!(expect, actual);
    }
}
