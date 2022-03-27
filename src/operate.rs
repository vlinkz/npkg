use std::{
    env, fs,
    process::{exit, Command},
};

pub enum OperateError {
    CmdError,
}

pub fn hminstall(packages: Vec<String>, currpkgs: Vec<String>) -> Result<(), OperateError> {
    let file = format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap());
    match cfginstall(packages, currpkgs, &file, "home.packages", "home-manager") {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn sysinstall(packages: Vec<String>, currpkgs: Vec<String>) -> Result<(), OperateError> {
    match cfginstall(
        packages,
        currpkgs,
        "/etc/nixos/configuration.nix",
        "environment.systemPackages",
        "nixos-rebuild",
    ) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn envinstall(packages: Vec<String>, currpkgs: Vec<String>) -> Result<(), OperateError> {
    let mut pkgs = vec![];

    for p in packages {
        if !currpkgs.contains(&p) {
            pkgs.push(format!("nixos.{}", p));
        }
    }

    if pkgs.is_empty() {
        println!("No new packages to install");
        exit(0);
    }

    match Command::new("nix-env")
        .arg("-iA")
        .arg(pkgs.join(" "))
        .status()
    {
        Ok(_) => Ok(()),
        Err(_) => Err(OperateError::CmdError),
    }
}

pub fn hmremove(packages: Vec<String>, currpkgs: Vec<String>) -> Result<(), OperateError> {
    let file = format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap());
    match cfgremove(packages, currpkgs, &file, "home.packages", "home-manager") {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn sysremove(packages: Vec<String>, currpkgs: Vec<String>) -> Result<(), OperateError> {
    match cfgremove(
        packages,
        currpkgs,
        "/etc/nixos/configuration.nix",
        "environment.systemPackages",
        "nixos-rebuild",
    ) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn envremove(packages: Vec<String>, currpkgs: Vec<String>) -> Result<(), OperateError> {
    let mut pkgs = vec![];

    for p in packages {
        if currpkgs.contains(&p) {
            pkgs.push(p);
        }
    }

    if pkgs.is_empty() {
        println!("No packages to remove");
        exit(0);
    }

    match Command::new("nix-env")
        .arg("-e")
        .arg(pkgs.join(" "))
        .status()
    {
        Ok(_) => Ok(()),
        Err(_) => Err(OperateError::CmdError),
    }
}

fn cfginstall(
    packages: Vec<String>,
    currpkgs: Vec<String>,
    file: &str,
    query: &str,
    cmd: &str,
) -> Result<(), OperateError> {
    let f = fs::read_to_string(file).expect("Failed to read file");

    //Add check for current packages
    let mut pkgs = vec![];
    for p in packages {
        if !currpkgs.contains(&p) {
            pkgs.push(p);
        }
    }

    if pkgs.is_empty() {
        println!("No new packages to install");
        exit(0);
    }

    let out = match nix_editor::write::addtoarr(&f, query, pkgs) {
        Ok(x) => x,
        Err(_) => exit(1),
    };

    fs::write(&file, out).expect("Unable to write file");

    let status = Command::new(cmd)
        .arg("switch")
        //.arg("--option")
        //.arg("substitute")
        //.arg("false")
        .status()
        .expect(&format!("Failed to run {}", cmd));

    if !status.success() {
        //printerror("Could not rebuild configuration");
        fs::write(&file, f).expect("Unable to write file");
        return Err(OperateError::CmdError);
    }
    return Ok(());
}

fn cfgremove(
    packages: Vec<String>,
    currpkgs: Vec<String>,
    file: &str,
    query: &str,
    cmd: &str,
) -> Result<(), OperateError> {
    let f = fs::read_to_string(file).expect("Failed to read file");

    //Add check for current packages
    let mut pkgs = vec![];
    for p in packages {
        if currpkgs.contains(&p) {
            pkgs.push(p);
        }
    }

    if pkgs.is_empty() {
        println!("No installed packages to remove");
        exit(0);
    }

    let out = match nix_editor::write::rmarr(&f, query, pkgs) {
        Ok(x) => x,
        Err(_) => exit(1),
    };

    fs::write(&file, out).expect("Unable to write file");

    let status = Command::new(cmd)
        .arg("switch")
        //.arg("--option")
        //.arg("substitute")
        //.arg("false")
        .status()
        .expect(&format!("Failed to run {}", cmd));

    if !status.success() {
        //printerror("Could not rebuild configuration");
        fs::write(&file, f).expect("Unable to write file");
        return Err(OperateError::CmdError);
    }
    return Ok(());
}
