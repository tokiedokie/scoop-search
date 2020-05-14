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
        let name = App::get_name(&path).unwrap();
        let (version, bin) = App::get_version_bin(&path).unwrap();
        App { name, version, bin }
    }

    pub fn get_name(path: &PathBuf) -> Result<String, Box<dyn Error>> {
        let name = path
            .file_stem()
            .ok_or("can't detect file name")?
            .to_os_string()
            .into_string()
            .map_err(|err| err.to_string_lossy().to_string())?;
        Ok(name)
    }

    pub fn get_version_bin(path: &Path) -> Result<(String, Vec<String>), Box<dyn Error>> {
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

    pub fn get_app_paths(bucket_path: &PathBuf) -> Vec<PathBuf> {
        let mut path: PathBuf = PathBuf::from(bucket_path);

        path.push("bucket");
        fs::read_dir(path)
            .unwrap()
            .map(|path| path.unwrap().path())
            .collect()
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

    pub fn search_remote_apps(remote_url: &str, query: &str) -> Vec<App> {
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
            .filter(|path| path.to_lowercase().contains(query))
            .map(|name| App {
                name,
                version: String::new(),
                bin: Vec::new(),
            })
            .collect();

        filtered
    }
}
