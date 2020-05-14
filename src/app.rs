use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub struct App {
    pub name: String,
    pub version: String,
    pub bin: Vec<String>,
}

impl App {
    pub fn new(path: &PathBuf) -> App {
        let name = App::get_name(&path).unwrap_or(String::from(""));
        let (version, bin) = App::get_version_bin(&path).unwrap_or((String::new(), Vec::new()));
        App { name, version, bin }
    }

    pub fn get_name(path: &PathBuf) -> Option<String> {
        let name = path.file_stem()?.to_os_string().into_string().ok()?;
        Some(name)
    }

    pub fn get_version_bin(path: &Path) -> Option<(String, Vec<String>)> {
        let manufest = fs::read_to_string(&path).ok()?;
        let manufest_json: serde_json::Value = serde_json::from_str(&manufest).ok()?;

        let version: String = match manufest_json.get("version") {
            Some(version) => version
                .as_str()
                .expect("version in manifest is invalid.")
                .to_string(),
            None => String::from(""),
        };

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

        Some((version, bin))
    }

    pub fn get_app_paths(bucket_path: &PathBuf) -> Option<Vec<PathBuf>> {
        let mut path: PathBuf = PathBuf::from(bucket_path);

        path.push("bucket");
        let app_paths: Vec<PathBuf> = fs::read_dir(path)
            .ok()?
            .map(|path| path.unwrap().path())
            .collect();

        if app_paths.is_empty() {
            return None;
        }

        Some(app_paths)
    }

    pub fn search_apps(apps: &Vec<App>, query: &str) -> Vec<App> {
        let mut result: Vec<App> = Vec::new();

        for app in apps {
            if app.name.to_lowercase().contains(query) {
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
                    if bin.to_lowercase().contains(query) {
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

    pub fn search_remote_apps(remote_url: &str, query: &str) -> Result<Vec<App>, Box<dyn Error>> {
        let response_json = ureq::get(remote_url).call().into_json()?;

        let tree = response_json
            .get("tree")
            .ok_or(Box::<dyn Error>::from("Can't get remote repository"))?;

        let filtered: Vec<App> = tree
            .as_array()
            .ok_or(Box::<dyn Error>::from(
                format!("{} key `tree` is invalid", remote_url),
            ))?
            .iter()
            .map(|obj| obj["path"].as_str().unwrap_or("").to_string())
            .filter(|path| path.ends_with(".json"))
            .map(|path| path.trim_end_matches(".json").to_string())
            .filter(|path| path.to_lowercase().contains(query))
            .map(|name| App {
                name,
                version: String::new(),
                bin: Vec::new(),
            })
            .collect();

        Ok(filtered)
    }
}
