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

    fn get_name(path: &PathBuf) -> String {
        let name = path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        name
    }

    fn get_bucket_paths(scoop: &Scoop) -> Vec<PathBuf> {
        let bucket_dirs = fs::read_dir(&scoop.buckets_dir).unwrap();

        bucket_dirs.map(|path| path.unwrap().path()).collect()
    }

    fn get_remote_names_urls(
        scoop: &Scoop,
        local_bucket_names: &Vec<String>,
    ) -> Vec<(String, String)> {
        let mut buckets_file = PathBuf::from(scoop.dir.as_os_str());
        buckets_file.push("apps\\scoop\\current\\buckets.json");

        let buckets_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&buckets_file).unwrap()).unwrap();
        let buckets_map = buckets_json.as_object().unwrap();

        let mut result: Vec<(String, String)> = Vec::new();

        for bucket_tuple in buckets_map {
            if local_bucket_names
                .iter()
                .find(|name| name == &bucket_tuple.0)
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
                result.push((bucket_tuple.0.clone(), api_link));
            }
        }
        result
    }

    fn search_local_buckets(bucket_paths: &Vec<PathBuf>, query: &str) -> Option<()> {
        let mut app_in_local = false;

        for bucket_path in bucket_paths {
            let bucket_name = Bucket::get_name(&bucket_path);
            let app_paths = App::get_app_paths(&bucket_path);

            let apps: Vec<App> = app_paths.iter().map(|path| App::new(path)).collect();

            let filtered_apps = App::search_apps(&apps, query);

            if filtered_apps.len() > 0 {
                app_in_local = true;
            }

            display_apps(&bucket_name, &filtered_apps);
        }

        if !app_in_local {
            return None;
        }

        Some(())
    }

    fn search_exclude_bin(bucket_paths: &Vec<PathBuf>, query: &str) -> Option<Vec<Bucket>> {
        let mut buckets: Vec<Bucket> = Vec::new();

        for bucket_path in bucket_paths {
            let bucket_name = Bucket::get_name(&bucket_path);
            let app_paths = App::get_app_paths(&bucket_path);

            let filtered_apps: Vec<App> = app_paths
                .iter()
                .filter(|app_path| App::get_name(app_path).contains(query))
                .map(|app_path| {
                    let name = App::get_name(app_path);
                    let (version, _) = App::get_version_bin(app_path).unwrap();
                    App {
                        name,
                        version,
                        bin: Vec::new(),
                    }
                })
                .collect();

            buckets.push(Bucket::new(bucket_name, filtered_apps))
        }

        if buckets
            .iter()
            .find(|bucket| bucket.apps.len() != 0)
            .is_some()
        {
            return Some(buckets);
        }

        None
    }

    fn search_remote_buckets(
        scoop: &Scoop,
        local_bucket_names: &Vec<String>,
        query: &str,
    ) -> Option<Vec<Bucket>> {
        let mut buckets: Vec<Bucket> = Vec::new();

        let remote_names_urls = Bucket::get_remote_names_urls(&scoop, &local_bucket_names);
        for remote_name_url in remote_names_urls {
            let remote_name = remote_name_url.0;
            let remote_url = remote_name_url.1;

            let remote_apps = App::search_remote_apps(&remote_url, query);

            buckets.push(Bucket::new(remote_name, remote_apps))
        }

        if buckets
            .iter()
            .find(|bucket| bucket.apps.len() != 0)
            .is_some()
        {
            return Some(buckets);
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
struct App {
    name: String,
    version: String,
    bin: Vec<String>,
}

impl App {
    fn new(path: &PathBuf) -> App {
        let name = App::get_name(&path);
        let (version, bin) = App::get_version_bin(&path).unwrap();
        App { name, version, bin }
    }

    fn get_name(path: &PathBuf) -> String {
        let name = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        name
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

    fn get_app_paths(bucket_path: &PathBuf) -> Vec<PathBuf> {
        let mut path: PathBuf = PathBuf::from(bucket_path);

        path.push("bucket");
        fs::read_dir(path)
            .unwrap()
            .map(|path| path.unwrap().path())
            .collect()
    }

    fn search_apps(apps: &Vec<App>, query: &str) -> Vec<App> {
        let mut result: Vec<App> = Vec::new();

        for app in apps {
            if app.name.contains(query) {
                result.push(App {
                    name: app.name.clone(),
                    version: app.version.clone(),
                    bin: Vec::new(),
                });
            } else {
                for bin in &app.bin {
                    let bin = Path::new(&bin)
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new(""))
                        .to_string_lossy()
                        .to_string();
                    if bin.contains(query) {
                        result.push(App {
                            name: app.name.clone(),
                            version: app.version.clone(),
                            bin: vec![bin],
                        })
                    }
                }
            }
        }

        result
    }

    fn search_remote_apps(remote_url: &str, query: &str) -> Vec<App> {
        let response_json = ureq::get(remote_url).call().into_json().unwrap();

        let tree = response_json
            .get("tree")
            .expect("Can't get remote repository");

        let filtered: Vec<App> = tree
            .as_array()
            .unwrap()
            .iter()
            .map(|obj| obj["path"].as_str().unwrap().to_string())
            .filter(|path| path.ends_with(".json"))
            .map(|path| path.trim_end_matches(".json").to_string())
            .filter(|path| path.contains(query))
            .map(|name| App {
                name,
                version: String::new(),
                bin: Vec::new(),
            })
            .collect();

        filtered
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
        let mut buckets_dir = PathBuf::from(dir.to_str().unwrap());
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

pub struct Args {
    pub query: String,
    pub exclude_bin: bool,
}

pub fn get_query(mut args: env::Args) -> Result<String, &'static str> {
    args.next();
    let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query"),
    };

    Ok(query.to_lowercase())
}

pub fn parse_args(mut args: env::Args) -> Result<Args, &'static str> {
    args.next();

    let query = match args.next() {
        Some(arg) => arg,
        None => return Err("Didn't get a query"),
    };

    let exclude_bin = args.find(|arg| arg == "-b").is_some();

    Ok(Args {
        query: query.to_lowercase(),
        exclude_bin,
    })
}

fn search_include_bin(scoop: &Scoop, query: &str) -> Result<(), Box<dyn Error>> {
    let bucket_paths = Bucket::get_bucket_paths(scoop);

    match Bucket::search_local_buckets(&bucket_paths, query) {
        Some(_) => {}
        None => {
            let local_bucket_names = &bucket_paths
                .iter()
                .map(|path| Bucket::get_name(path))
                .collect();
            match Bucket::search_remote_buckets(scoop, local_bucket_names, query) {
                Some(buckets) => {
                    println!("Results from other known buckets...");
                    println!("(add them using 'scoop bucket add <name>')");
                    println!("");
                    display_buckets(&buckets);
                }
                None => println!("No matches found."),
            }
        }
    }

    Ok(())
}

fn search_exclude_bin(scoop: &Scoop, query: &str) -> Result<(), Box<dyn Error>> {
    let bucket_paths = Bucket::get_bucket_paths(scoop);

    match Bucket::search_exclude_bin(&bucket_paths, query) {
        Some(buckets) => display_buckets(&buckets),
        None => {
            let local_bucket_names = &bucket_paths
                .iter()
                .map(|path| Bucket::get_name(path))
                .collect();
            match Bucket::search_remote_buckets(scoop, local_bucket_names, query) {
                Some(buckets) => {
                    println!("Results from other known buckets...");
                    println!("(add them using 'scoop bucket add <name>')");
                    println!("");
                    display_buckets(&buckets);
                }
                None => match Bucket::search_local_buckets(&bucket_paths, query) {
                    Some(_) => {}
                    None => println!("No matches found."),
                },
            }
        }
    }

    Ok(())
}

pub fn run(scoop: &Scoop, query: &str) -> Result<(), Box<dyn Error>> {
    search_exclude_bin(scoop, query)
}

fn display_apps(bucket_name: &str, apps: &Vec<App>) {
    if apps.len() > 0 {
        println!("'{}' bucket: ", bucket_name,);
        for app in apps {
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

fn display_buckets(buckets: &Vec<Bucket>) {
    for bucket in buckets {
        display_apps(&bucket.name, &bucket.apps);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search_apps() {
        let apps = vec![App {
            name: String::from("test_app"),
            version: String::from("test_version"),
            bin: vec![String::from("test_bin")],
        }];
        let query = String::from("test");

        let expect = vec![App {
            name: String::from("test_app"),
            version: String::from("test_version"),
            bin: Vec::new(),
        }];

        let actual = App::search_apps(&apps, &query);

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_search_remote_apps() {
        let remote_url =
            "https://api.github.com/repos/ScoopInstaller/Main/git/trees/HEAD?recursive=1";
        let query = "7zip";
        let actual = App::search_remote_apps(remote_url, query);

        let expect = vec![App {
            name: String::from("bucket/7zip"),
            version: String::new(),
            bin: Vec::new(),
        }];

        assert_eq!(expect, actual);
    }

    #[test]
    fn test_search_exclude_bin() {
        let scoop = Scoop::new();
        let bucket_paths = Bucket::get_bucket_paths(&scoop);
        let query = "7zip";

        let actual = Bucket::search_exclude_bin(&bucket_paths, query);

        let expect = Some(vec![
            Bucket {
                name: String::from("extras"),
                apps: Vec::new(),
            },
            Bucket {
                name: String::from("games"),
                apps: Vec::new(),
            },
            Bucket {
                name: String::from("java"),
                apps: Vec::new(),
            },
            Bucket {
                name: String::from("main"),
                apps: vec![App {
                    name: String::from("7zip"),
                    version: String::from("19.00"),
                    bin: Vec::new(),
                }],
            },
            Bucket {
                name: String::from("nerd-fonts"),
                apps: Vec::new(),
            },
        ]);

        assert_eq!(expect, actual);
    }
}
