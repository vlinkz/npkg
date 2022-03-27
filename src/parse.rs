use std::{env, fs, process::Command};
use serde_json::{self, Value};
pub enum ParseError {
    EmptyPkgs,
}

pub fn hmpkgs() -> Result<Vec<String>,ParseError> {
    let file = format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap());
    let f = fs::read_to_string(&file).expect("Failed to read file");

    //Add check for current packages
    let currpkgs = match nix_editor::read::getarrvals(&f, "home.packages") {
        Ok(x) => x,
        Err(_) => {
            return Err(ParseError::EmptyPkgs);
        }
    };
    return Ok(currpkgs);
}

pub fn syspkgs() -> Result<Vec<String>,ParseError> {
    let file = "/etc/nixos/configuration.nix";
    let f = fs::read_to_string(file).expect("Failed to read file");

    //Add check for current packages
    let currpkgs = match nix_editor::read::getarrvals(&f, "environment.systemPackages") {
        Ok(x) => x,
        Err(_) => {
            return Err(ParseError::EmptyPkgs);
        }
    };
    return Ok(currpkgs);
}

pub fn envpkgs() -> Result<Vec<String>,ParseError> {

    let out = Command::new("nix-env")
        .arg("-q")
        .arg("--json")
        .output()
        .expect("Failed to execute process");

    let data: Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("Failed to parse json");

    let mut currpkgs = vec![];
    for (_,pkg) in data.as_object().unwrap() {
        currpkgs.push(pkg.as_object().unwrap()["pname"].as_str().unwrap().to_string());
    }

    return Ok(currpkgs);
}