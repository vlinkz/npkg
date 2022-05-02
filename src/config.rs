use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};
use owo_colors::*;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    systemconfig: String,
    homeconfig: String,
    flake: Option<String>,
}

pub fn checkconfig() -> String {
    let cfgdir = format!("{}/.config/npkg", env::var("HOME").unwrap());
    if !Path::is_file(Path::new(&format!("{}/config.json", &cfgdir))) {
        if !Path::is_file(Path::new(&format!("/etc/npkg/config.json"))) {
            createconfig();
            return cfgdir;
        } else {
            return "/etc/npkg/".to_string();
        }
    } else {
        return cfgdir;
    }
}

fn createconfig() {
    let cfgdir = format!("{}/.config/npkg", env::var("HOME").unwrap());
    fs::create_dir_all(&cfgdir).expect("Failed to create config directory");
    let config = Config {
        systemconfig: "/etc/nixos/configuration.nix".to_string(),
        homeconfig: format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap()),
        flake: None,
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create(format!("{}/config.json", cfgdir)).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn readconfig(cfgdir: String) -> (String, String, Option<String>) {
    let file = fs::read_to_string(format!("{}/config.json", cfgdir)).unwrap();
    let config: Config = match serde_json::from_str(&file) {
        Ok(x) => x,
        Err(e) => {
            println!("{} {}","Failed to parse config:".red(), e);
            println!("Using default values");
            return (
                "/etc/nixos/configuration.nix".to_string(),
                format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap()),
                None,
            );
        }
    };
    if Path::is_file(Path::new(&config.systemconfig)) {
        return (config.systemconfig, config.homeconfig, config.flake);
    } else {
        println!("{}", "Config file is invalid".bright_red());
        println!("{}", "Using default values".yellow());
        return (
            "/etc/nixos/configuration.nix".to_string(),
            format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap()),
            None,
        );
    }
}
