use crate::PkgData;
use brotli;
use curl::easy::Easy;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::Command,
};

//use simd_json::{self};
#[derive(Serialize, Deserialize, Debug)]
struct PackageBase {
    //#[serde(rename = "type")]
    packages: HashMap<String, Package>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    pname: String,
    version: String,
    meta: Meta,
}
#[derive(Serialize, Deserialize, Debug)]
struct Meta {
    broken: Option<bool>,
    description: Option<String>,
}

pub fn search(query: &Vec<String>) -> Result<Vec<PkgData>, String> {
    checkcache();

    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
    let file = fs::read_to_string(format!("{}/packages.json", cachedir)).unwrap();
    //println!("Read file");
    let data: PackageBase = serde_json::from_str(&file).expect("Failed to parse json");
    let pkgs = data.packages.keys().filter(|x| {
        let mut b = true;
        let desc = &data.packages.get(&x.to_string()).unwrap().meta.description;
        for q in query {
            let desc = match desc {
                Some(y) => y.to_lowercase().contains(q.to_lowercase().as_str()),
                None => false,
            };
            b &= x.to_lowercase().contains(q.to_lowercase().as_str()) || desc;
        }
        b
    });

    let mut out = vec![];
    for i in pkgs {
        let pkg = data.packages.get(i).unwrap();
        out.push(PkgData {
            pname: i.to_string(),
            description: pkg.meta.description.clone(),
            version: pkg.version.clone(),
        });
    }

    out.sort_by_key(|x| x.pname.clone());
    return Ok(out);
}

fn checkcache() {
    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());

    let vout = Command::new("nixos-version")
        .arg("--json")
        .output()
        .unwrap();
    let data: Value =
        serde_json::from_str(&String::from_utf8_lossy(&vout.stdout)).expect("Failed to parse json");

    let version = data.as_object().unwrap()["nixosVersion"].as_str().unwrap();
    //println!("Version is {}", version);

    if !Path::is_dir(Path::new(&cachedir))
        || !Path::is_file(Path::new(&format!("{}/version.json", &cachedir)))
    {
        setupcache(version);
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir)).unwrap();
        newver.write_all(&vout.stdout).unwrap();
    }

    let file = fs::read_to_string(format!("{}/version.json", cachedir)).unwrap();
    let olddata: Value = serde_json::from_str(&file).expect("Failed to parse json");

    let oldversion = olddata.as_object().unwrap()["nixosVersion"]
        .as_str()
        .unwrap();

    //println!("Old Version is {}", version);

    if version != oldversion {
        println!("Out of date, updating cache");
        setupcache(version);
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir)).unwrap();
        newver.write_all(&vout.stdout).unwrap();
    } else if !Path::is_file(Path::new(&format!("{}/packages.json", &cachedir))) {
        println!("No packages.json, updating cache");
        setupcache(version);
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir)).unwrap();
        newver.write_all(&vout.stdout).unwrap();
    }
}

fn setupcache(version: &str) {
    let mut relver = version.split(".").collect::<Vec<&str>>()[0..2].join(".");
    if &relver[0..5] == "22.05" {
        relver = "unstable".to_string();
    }
    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
    fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
    let url = format!(
        "https://releases.nixos.org/nixos/{}/nixos-{}/packages.json.br",
        relver, version
    );

    if Path::is_file(Path::new(&format!("{}/packages.json", &cachedir))) {
        fs::remove_file(Path::new(&format!("{}/packages.json", &cachedir)))
            .expect("Failed to remove file");
    }

    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(&url).unwrap();

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    {
        let mut file = File::create(format!("{}/packages.json.br", &cachedir).as_str())
            .expect("Failed to create file");
        file.write_all(dst.as_slice())
            .expect("Failed to write file");
    }

    {
        let mut file = File::create(format!("{}/packages.json", &cachedir).as_str())
            .expect("Failed to create file");
        let mut reader = brotli::Decompressor::new(
            dst.as_slice(),
            4096, // buffer size
        );
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf[..]) {
                Err(e) => {
                    if let std::io::ErrorKind::Interrupted = e.kind() {
                        continue;
                    }
                    panic!("{}", e);
                }
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    match file.write_all(&buf[..size]) {
                        Err(e) => panic!("{}", e),
                        Ok(_) => {}
                    }
                }
            }
        }
    }
}
