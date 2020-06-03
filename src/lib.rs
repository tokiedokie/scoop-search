use std::env;
use std::error::Error;

pub mod scoop;
use self::scoop::Scoop;

mod bucket;
use self::bucket::Bucket;

mod app;
use self::app::App;

#[derive(Debug, std::cmp::PartialEq)]
pub struct Args {
    pub query: String,
    pub exclude_bin: bool,
}

pub fn parse_args(args: env::Args) -> Result<Args, &'static str> {
    let args: Vec<String> = args.collect();
    let query: String;
    let mut exclude_bin = true;

    match &args.len() {
        1 => query = String::new(),
        2 => {
            if args[1] == "--bin" {
                exclude_bin = false;
                query = String::new();
            } else if args[1] == "*" {
                query = String::new();
            } else {
                query = args[1].clone();
            }
        }
        3 => {
            if args[1] == "--bin" {
                exclude_bin = false;
                if args[2] == "*" {
                    query = String::new();
                } else {
                    query = args[2].clone();
                }
            } else {
                return Err("option is not valid");
            }
        }
        _ => return Err("args number incorrect."),
    }

    Ok(Args {
        query: query.to_lowercase(),
        exclude_bin,
    })
}

///this function display result so you don't need to treat it.
fn search_include_bin(scoop: &Scoop, query: &str) -> Result<(), Box<dyn Error>> {
    let bucket_paths = Bucket::get_bucket_paths(scoop);

    if Bucket::search_include_bin(&bucket_paths, query).is_none() {
        let local_bucket_names = &bucket_paths
            .iter()
            .map(|path| Bucket::get_name(path).unwrap_or_default())
            .collect();
        match Bucket::search_remote_buckets(scoop, local_bucket_names, query) {
            Some(buckets) => {
                println!("Results from other known buckets...");
                println!("(add them using 'scoop bucket add <name>')");
                println!();
                display_buckets(&buckets);
            }
            None => println!("No matches found."),
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
                .map(|path| Bucket::get_name(path).unwrap_or_default())
                .collect();
            match Bucket::search_remote_buckets(scoop, local_bucket_names, query) {
                Some(buckets) => {
                    println!("Results from other known buckets...");
                    println!("(add them using 'scoop bucket add <name>')");
                    println!();
                    display_buckets(&buckets);
                }
                None => match Bucket::search_include_bin(&bucket_paths, query) {
                    Some(_) => {}
                    None => println!("No matches found."),
                },
            }
        }
    }

    Ok(())
}

pub fn run(scoop: &Scoop, args: &Args) -> Result<(), Box<dyn Error>> {
    if args.exclude_bin {
        search_exclude_bin(scoop, &args.query)
    } else {
        search_include_bin(scoop, &args.query)
    }
}

fn display_apps(bucket_name: &str, apps: &Vec<App>) {
    if !apps.is_empty() {
        println!("'{}' bucket: ", bucket_name,);
        for app in apps {
            if app.version != "" {
                if !app.bin.is_empty() {
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
        println!();
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
        let actual = App::search_remote_apps(remote_url, query).unwrap();

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
