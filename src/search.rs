use crate::PkgData;
use bimap;
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

#[derive(Serialize, Deserialize, Debug)]
struct PackageBase {
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

pub fn pname_to_name(query: &Vec<String>) -> Vec<String> {
    checkcache();

    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
    let file = fs::read_to_string(format!("{}/pnameref.json", cachedir)).unwrap();

    let data: bimap::BiHashMap<String, String> = serde_json::from_str(&file).expect("Failed to parse json");

    let mut pkgs = vec![];
    for q in query {
        pkgs.push(match data.get_by_left(q) {
            Some(x) => x.to_string(),
            None => q.to_string(),
        });
    }
    return pkgs;
}

pub fn name_to_pname(query: &Vec<String>) -> Vec<String> {
    checkcache();

    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
    let file = fs::read_to_string(format!("{}/pnameref.json", cachedir)).unwrap();
    let data: bimap::BiHashMap<String, String> = serde_json::from_str(&file).expect("Failed to parse json");
    let mut pkgs = vec![];
    for q in query {
        let n = match data.get_by_right(q) {
            Some(x) => x.to_string(),
            None => q.to_string(),
        };
        pkgs.push(n);
    }
    return pkgs;
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

    if !Path::is_dir(Path::new(&cachedir))
        || !Path::is_file(Path::new(&format!("{}/version.json", &cachedir)))
    {
        println!("Updating cache");
        setupcache(version);
        let mut newver = fs::File::create(format!("{}/version.json", &cachedir)).unwrap();
        newver.write_all(&vout.stdout).unwrap();
    }

    let file = fs::read_to_string(format!("{}/version.json", cachedir)).unwrap();
    let olddata: Value = serde_json::from_str(&file).expect("Failed to parse json");

    let oldversion = olddata.as_object().unwrap()["nixosVersion"]
        .as_str()
        .unwrap();

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
    } else if !Path::is_file(Path::new(&format!("{}/pnameref.json", &cachedir))) {
        println!("Updating references");
        updatepnameref();
    }
}

fn setupcache(version: &str) {
    let mut relver = version.split(".").collect::<Vec<&str>>()[0..2].join(".");
    if &relver[0..5] == "22.05" {
        relver = "unstable".to_string();
    }

    let mut dlver = version.to_string();
    let v = version.split(".").collect::<Vec<&str>>();
    if v.len() == 4 {
        if v[3].len() == 7 {
            println!("Detected system using flakes: finding proper nixos version");
            let vout = Command::new("nix-instantiate")
                .arg("-E")
                .arg("with import <nixpkgs/lib>; version")
                .arg("--eval")
                .arg("--json")
                .output()
                .unwrap();
            dlver = String::from_utf8_lossy(&vout.stdout).to_string().replace("\"", ""); 
        }
    }

    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
    fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
    let url = format!(
        "https://releases.nixos.org/nixos/{}/nixos-{}/packages.json.br",
        relver, dlver
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
    updatepnameref();
}

fn updatepnameref() {
    let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
    let file = fs::read_to_string(format!("{}/packages.json", cachedir)).unwrap();
    let data: PackageBase = serde_json::from_str(&file).expect("Failed to parse json");
    let mut hmap = bimap::BiHashMap::new();
    for (s, pkg) in data.packages {
        hmap.insert(s, pkg.name.to_string());
    }
    let out = match serde_json::to_string(&hmap) {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };
    let mut outfile = File::create(format!("{}/pnameref.json", &cachedir).as_str())
        .expect("Failed to create file");
    outfile.write_all(out.as_bytes())
        .expect("Failed to write file");
}