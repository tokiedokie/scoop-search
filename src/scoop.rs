use std::env;
use std::error::Error;
use std::fs;
use std::path::{PathBuf};

#[derive(Debug, PartialEq)]
pub struct Scoop {
    pub dir: PathBuf,
    pub buckets_dir: PathBuf,
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
