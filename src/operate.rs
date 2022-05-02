use crate::{search::pname_to_name, NpkgData};
use owo_colors::*;
use std::{
    fs,
    path::Path,
    process::{exit, Command}, env,
};

pub enum OperateError {
    CmdError,
    WriteError(String),
}

enum Actions {
    Install,
    Remove,
}

pub fn pkinstall(opts: NpkgData) -> Result<(), OperateError> {
    match cfgoperate(opts, Actions::Install) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn pkremove(opts: NpkgData) -> Result<(), OperateError> {
    match cfgoperate(opts, Actions::Remove) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn chnupdate(opts: &NpkgData) {
    println!("{}", "Updating channels...".green());
    let _userchannel = Command::new("nix-channel")
        .arg("--update")
        .status()
        .expect("Failed to execute process nix-channel");
    println!(
        "{}",
        "Need root access to update system channels".bright_magenta()
    );
    let _syschannel = Command::new("sudo")
        .arg("nix-channel")
        .arg("--update")
        .status()
        .expect("Failed to execute process nix-channel");
    if opts.flake.is_some() {
        println!("{}", "Updating flake...".green());
        let _flakechannel = Command::new("nix")
            .arg("flake")
            .arg("update")
            .arg(
                opts.flake
                    .as_ref()
                    .unwrap()
                    .split("#")
                    .collect::<Vec<&str>>()[0],
            )
            .status()
            .expect("Failed to execute process nix flake");
    }
}

pub fn envinstall(opts: NpkgData) -> Result<(), OperateError> {
    let mut pkgs = vec![];

    for p in opts.pkgs {
        if !opts.currpkgs.contains(&p) {
            pkgs.push(format!("nixos.{}", p));
        }
    }

    if pkgs.is_empty() {
        println!("No new packages to install");
        exit(0);
    }

    match Command::new("nix-env").arg("-iA").args(pkgs).status() {
        Ok(_) => Ok(()),
        Err(_) => Err(OperateError::CmdError),
    }
}

pub fn envremove(opts: NpkgData) -> Result<(), OperateError> {
    let mut pkgs = vec![];

    for p in opts.pkgs {
        if opts.currpkgs.contains(&p) {
            pkgs.push(p);
        }
    }

    if pkgs.is_empty() {
        println!("No packages to remove");
        exit(0);
    }

    pkgs = pname_to_name(&pkgs);

    match Command::new("nix-env").arg("-e").args(pkgs).status() {
        Ok(_) => Ok(()),
        Err(_) => Err(OperateError::CmdError),
    }
}

pub fn envupdate() -> Result<(), OperateError> {
    match Command::new("nix-env").arg("-u").arg("*").status() {
        Ok(_) => Ok(()),
        Err(_) => Err(OperateError::CmdError),
    }
}

fn cfgoperate(mut opts: NpkgData, action: Actions) -> Result<(), OperateError> {
    match &opts.output {
        Some(_) => {
            opts.dryrun = true;
        }
        None => {}
    };

    let f = match opts.pkgmgr {
        crate::PackageTypes::Home => fs::read_to_string(&opts.hmcfg).expect("Failed to read file"),
        crate::PackageTypes::System => {
            fs::read_to_string(&opts.syscfg).expect("Failed to read file")
        }
        _ => {
            println!("{}", "Unsupported package type".red());
            exit(1);
        }
    };

    //Add check for current packages
    let mut pkgs = vec![];
    for p in &opts.pkgs {
        match action {
            Actions::Install => {
                if !opts.currpkgs.contains(&p) {
                    pkgs.push(p.to_string());
                }
            }
            Actions::Remove => {
                if opts.currpkgs.contains(&p) {
                    pkgs.push(p.to_string());
                }
            }
        }
    }

    if pkgs.is_empty() {
        println!("No new packages to install");
        exit(0);
    }

    let query = match opts.pkgmgr {
        crate::PackageTypes::Home => "home.packages",
        crate::PackageTypes::System => "environment.systemPackages",
        _ => {
            println!("{}", "Unsupported package type".red());
            exit(1);
        }
    };

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

    let out = match action {
        Actions::Install => match nix_editor::write::addtoarr(&f, query, pkgs) {
            Ok(x) => x,
            Err(_) => exit(1),
        },
        Actions::Remove => match nix_editor::write::rmarr(&f, query, pkgs) {
            Ok(x) => x,
            Err(_) => exit(1),
        },
    };

    let outfile = match opts.output {
        Some(ref s) => s.to_string(),
        None => match opts.pkgmgr {
            crate::PackageTypes::Home => opts.hmcfg.to_string(),
            crate::PackageTypes::System => opts.syscfg.to_string(),
            _ => {
                println!("{}", "Unsupported package type".red());
                exit(1);
            }
        },
    };

    match fs::write(&outfile, &out) {
        Ok(_) => {}
        Err(_) => {
            if Path::new(&outfile).is_file() {

                println!("{} {}", "Root permissions needed to modify".bright_yellow(), &outfile.green());

                let file = outfile.split("/").collect::<Vec<&str>>().pop().unwrap();
                let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
                fs::create_dir_all(&cachedir).expect("Failed to create cache directory");

                match fs::write(format!("{}/{}", cachedir, file), &out) {
                    Ok(_) => {
                        Command::new("sudo")
                            .arg("cp")
                            .arg(format!("{}/{}", cachedir, file))
                            .arg(&outfile)
                            .status()
                            .expect(&format!("{}", "Failed to execute process sudo cp".red()));
                        fs::remove_file(format!("{}/{}", cachedir, file)).expect("Failed to remove file");
                    }
                    Err(_) => {
                        let mut dir = outfile.split("/").collect::<Vec<&str>>();
                        dir.pop();
                        return Err(OperateError::WriteError(dir.join("/")));
                    }
                };
            } else {
                let mut dir = outfile.split("/").collect::<Vec<&str>>();
                dir.pop();
                return Err(OperateError::WriteError(dir.join("/")));
            }
        }
    };

    if !opts.dryrun {
        match cfgswitch(&opts) {
            Ok(()) => {}
            Err(e) => {
                println!("{}", "Failed to switch config".red());
                match fs::write(&outfile, &f) {
                    Ok(_) => {}
                    Err(_) => {
                        if Path::new(&outfile).is_file() {
                            
                            println!("{} {}", "Root permissions needed to restore".bright_yellow(), &outfile.green());
            
                            let file = outfile.split("/").collect::<Vec<&str>>().pop().unwrap();
                            let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
                            fs::create_dir_all(&cachedir).expect("Failed to create cache directory");
            
                            match fs::write(format!("{}/{}", cachedir, file), &f) {
                                Ok(_) => {
                                    Command::new("sudo")
                                        .arg("cp")
                                        .arg(format!("{}/{}", cachedir, file))
                                        .arg(&outfile)
                                        .status()
                                        .expect(&format!("{}", "Failed to execute process sudo cp".red()));
                                    fs::remove_file(format!("{}/{}", cachedir, file)).expect("Failed to remove file");
                                }
                                Err(_) => {
                                    let mut dir = outfile.split("/").collect::<Vec<&str>>();
                                    dir.pop();
                                    return Err(OperateError::WriteError(dir.join("/")));
                                }
                            };
                        } else {
                            let mut dir = outfile.split("/").collect::<Vec<&str>>();
                            dir.pop();
                            return Err(OperateError::WriteError(dir.join("/")));
                        }
                    }
                };
                return Err(e);
            }
        }
    }
    return Ok(());
}

pub fn cfgswitch(opts: &NpkgData) -> Result<(), OperateError> {
    let cmd = match opts.pkgmgr {
        crate::PackageTypes::Home => "home-manager".to_string(),
        crate::PackageTypes::System => "nixos-rebuild".to_string(),
        _ => {
            println!("{}", "Unsupported package type".red());
            exit(1);
        }
    };

    let status = match &opts.flake {
        None => match &opts.pkgmgr {
            crate::PackageTypes::System => {
                println!("{}", "Need root access to rebuild system".bright_magenta());
                Command::new(&cmd)
                    .arg("switch")
                    .arg("--use-remote-sudo")
                    .status()
                    .expect(&format!("Failed to run {}", &cmd))
            }
            _ => Command::new(&cmd)
                .arg("switch")
                .status()
                .expect(&format!("Failed to run {}", &cmd)),
        },
        Some(s) => {
            println!("Rebuilding with nix flakes");
            match &opts.pkgmgr {
                crate::PackageTypes::System => {
                    println!("{}", "Need root access to rebuild system".bright_magenta());
                    Command::new(&cmd)
                        .arg("switch")
                        .arg("--flake")
                        .arg(s)
                        .arg("--use-remote-sudo")
                        .status()
                        .expect(&format!("Failed to run {}", &cmd))
                }
                _ => Command::new(&cmd)
                    .arg("switch")
                    .arg("--flake")
                    .arg(s)
                    .status()
                    .expect(&format!("Failed to run {}", &cmd)),
            }
        }
    };

    if !status.success() {
        return Err(OperateError::CmdError);
    }

    Ok(())
}
