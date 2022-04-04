use crate::search::name_to_pname;
use std::{
    env, fs,
    process::{exit, Command},
};

pub enum OperateError {
    CmdError,
    WriteError(String),
}

enum Actions {
    Install,
    Remove,
}

pub fn hminstall(
    file: String,
    packages: Vec<String>,
    currpkgs: Vec<String>,
    output: Option<String>,
    build: bool,
) -> Result<(), OperateError> {
    let mut b = build;
    //let file = format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap());
    let outfile = match output {
        Some(s) => {
            b = false;
            s
        }
        None => file.to_string(),
    };
    match cfgoperate(
        packages,
        currpkgs,
        &file,
        &outfile,
        "home.packages",
        Actions::Install,
        "home-manager",
        b,
    ) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn sysinstall(
    file: String,
    packages: Vec<String>,
    currpkgs: Vec<String>,
    output: Option<String>,
    build: bool,
) -> Result<(), OperateError> {
    let mut b = build;
    //let file = "/etc/nixos/configuration.nix".to_string();
    let outfile = match output {
        Some(s) => {
            b = false;
            s
        }
        None => file.to_string(),
    };
    match cfgoperate(
        packages,
        currpkgs,
        &file,
        &outfile,
        "environment.systemPackages",
        Actions::Install,
        "nixos-rebuild",
        b,
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

pub fn hmremove(
    file: String,
    packages: Vec<String>,
    currpkgs: Vec<String>,
    output: Option<String>,
    build: bool,
) -> Result<(), OperateError> {
    //let file = format!("{}/.config/nixpkgs/home.nix", env::var("HOME").unwrap());
    let mut b = build;
    let outfile = match output {
        Some(s) => {
            b = false;
            s
        }
        None => file.to_string(),
    };
    match cfgoperate(
        packages,
        currpkgs,
        &file,
        &outfile,
        "home.packages",
        Actions::Remove,
        "home-manager",
        b,
    ) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn sysremove(
    file: String,
    packages: Vec<String>,
    currpkgs: Vec<String>,
    output: Option<String>,
    build: bool,
) -> Result<(), OperateError> {
    let mut b = build;
    //let file = "/etc/nixos/configuration.nix".to_string();
    let outfile = match output {
        Some(s) => {
            b = false;
            s
        }
        None => file.to_string(),
    };

    match cfgoperate(
        packages,
        currpkgs,
        &file,
        &outfile,
        "environment.systemPackages",
        Actions::Remove,
        "nixos-rebuild",
        b,
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

    pkgs = name_to_pname(&pkgs);

    match Command::new("nix-env")
        .arg("-e")
        .arg(pkgs.join(" "))
        .status()
    {
        Ok(_) => Ok(()),
        Err(_) => Err(OperateError::CmdError),
    }
}

fn cfgoperate(
    packages: Vec<String>,
    currpkgs: Vec<String>,
    file: &str,
    outfile: &str,
    query: &str,
    installorrm: Actions,
    cmd: &str,
    build: bool,
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

    let pkgsset = match nix_editor::read::getwithvalue(&f, query) {
        Ok(s) => s.contains(&"pkgs".to_string()),
        Err(_) => false,
    };

    if !pkgsset {
        pkgs = pkgs
            .into_iter()
            .map(|x| format!("pkgs.{}", x))
            .collect::<Vec<String>>();
    }

    /*let out = if installorrm {
        match nix_editor::write::addtoarr(&f, query, pkgs) {
            Ok(x) => x,
            Err(_) => exit(1),
        }
    } else {
        match nix_editor::write::rmarr(&f, query, pkgs) {
            Ok(x) => x,
            Err(_) => exit(1),
        }
    };*/

    let out = match installorrm {
        Actions::Install => match nix_editor::write::addtoarr(&f, query, pkgs) {
            Ok(x) => x,
            Err(_) => exit(1),
        },
        Actions::Remove => match nix_editor::write::rmarr(&f, query, pkgs) {
            Ok(x) => x,
            Err(_) => exit(1),
        },
    };

    match fs::write(&outfile, &out) {
        Ok(_) => {}
        Err(_) => {
            let mut file = outfile.split("/").collect::<Vec<&str>>();
            file.pop();
            return Err(OperateError::WriteError(file.join("/")));
        }
    };

    if build {
        let status = Command::new(cmd)
            .arg("switch")
            //.arg("--option")
            //.arg("substitute")
            //.arg("false")
            .status()
            .expect(&format!("Failed to run {}", cmd));

        if !status.success() {
            //printerror("Could not rebuild configuration");
            fs::write(&outfile, f).expect("Unable to write file");
            return Err(OperateError::CmdError);
        }
    }
    return Ok(());
}
