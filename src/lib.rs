use std::env;
use std::fs;
use std::path::PathBuf;

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

    Ok(query)
}

fn get_scoop_dir() -> PathBuf {
    // Todo: original scoop search $env:SCOOP, (get_config 'rootPath'), "$env:USERPROFILE\scoop"
    // so this is not enough.
    let mut userprofile = PathBuf::from(env::var("USERPROFILE").unwrap());
    userprofile.push("scoop");
    userprofile
}

pub fn get_bucket(scoop: &Scoop, query: &str) {
    //println!("{}", scoop.buckets_dir);

    let buckets = fs::read_dir(&scoop.buckets_dir).unwrap();

    for bucket in buckets {
        let mut bucket = bucket.unwrap().path();
        bucket.push("bucket");

        let apps = fs::read_dir(bucket).unwrap();
        /*
        let app = apps.filter(|filename| filename.as_ref().unwrap().file_name() == "games");
        println!("{:?}", app);
        */
        for app in apps {
            let file_name_osstr = app.unwrap().file_name();
            let file_name = file_name_osstr.to_str().unwrap();
            //println!("{:?}", &app);

            if file_name.contains(query) {
                println!("{}", file_name);
            }
        }
    }
}
