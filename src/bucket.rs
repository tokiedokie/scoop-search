use std::fs;
use std::path::PathBuf;

use crate::app::App;
use crate::scoop::Scoop;

#[derive(Debug, PartialEq)]
pub struct Bucket {
    pub name: String,
    pub apps: Vec<App>,
}

impl Bucket {
    fn new(name: String, apps: Vec<App>) -> Bucket {
        Bucket { name, apps }
    }

    pub fn get_name(path: &PathBuf) -> String {
        let name = path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        name
    }

    pub fn get_bucket_paths(scoop: &Scoop) -> Vec<PathBuf> {
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

    pub fn search_local_buckets(bucket_paths: &Vec<PathBuf>, query: &str) -> Option<()> {
        let mut app_in_local = false;

        for bucket_path in bucket_paths {
            let bucket_name = Bucket::get_name(&bucket_path);
            let app_paths = App::get_app_paths(&bucket_path);

            let apps: Vec<App> = app_paths.iter().map(|path| App::new(path)).collect();

            let filtered_apps = App::search_apps(&apps, query);

            if filtered_apps.len() > 0 {
                app_in_local = true;
            }

            crate::display_apps(&bucket_name, &filtered_apps);
        }

        if !app_in_local {
            return None;
        }

        Some(())
    }

    pub fn search_exclude_bin(bucket_paths: &Vec<PathBuf>, query: &str) -> Option<Vec<Bucket>> {
        let mut buckets: Vec<Bucket> = Vec::new();

        for bucket_path in bucket_paths {
            let bucket_name = Bucket::get_name(&bucket_path);
            let app_paths = App::get_app_paths(&bucket_path);

            let filtered_apps: Vec<App> = app_paths
                .iter()
                .filter(|app_path| App::get_name(app_path).to_lowercase().contains(query))
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

    pub fn search_remote_buckets(
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
