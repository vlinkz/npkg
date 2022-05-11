use crate::npkgcmd::search::name_to_pname;
use serde_json::{self, Value};
use std::{fs, process::Command};
pub enum ParseError {
    EmptyPkgs,
}

pub fn hmpkgs(file: String) -> Result<Vec<String>, ParseError> {
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

pub fn syspkgs(file: String) -> Result<Vec<String>, ParseError> {
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

pub fn envpkgs() -> Result<Vec<String>, ParseError> {
    let out = Command::new("nix-env")
        .arg("-q")
        .arg("--json")
        .output()
        .expect("Failed to execute process");

    let data: Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("Failed to parse json");

    let mut pcurrpkgs = vec![];
    for (_, pkg) in data.as_object().unwrap() {
        pcurrpkgs.push(
            pkg.as_object().unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    }

    let currpkgs = name_to_pname(&pcurrpkgs);

    return Ok(currpkgs);
}
