use crate::npkgcmd::{search::pname_to_name, NpkgData};
use npkg::*;
use owo_colors::*;
use std::{
    env, fs,
    path::Path,
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
    updatechannel().expect_err("Failed to execute process nix-channel");
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
        updateflake(opts.flake.as_ref().unwrap()).expect_err("Failed to execute process nix flake");
    }
}

pub fn envinstall_check(opts: NpkgData) -> Result<(), OperateError> {
    let mut pkgs = vec![];

    for p in opts.pkgs {
        if !opts.currpkgs.contains(&p) {
            pkgs.push(p);
        }
    }

    if pkgs.is_empty() {
        println!("No new packages to install");
        exit(0);
    }

    match envinstall(pkgs) {
        Ok(()) => Ok(()),
        Err(ExecuteError::WriteError(e)) => Err(OperateError::WriteError(e)),
        Err(ExecuteError::CmdError) => Err(OperateError::CmdError),
    }
}

pub fn envremove_check(opts: NpkgData) -> Result<(), OperateError> {
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

    match envremove(pkgs) {
        Ok(()) => Ok(()),
        Err(ExecuteError::WriteError(e)) => Err(OperateError::WriteError(e)),
        Err(ExecuteError::CmdError) => Err(OperateError::CmdError),
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
        crate::npkgcmd::PackageTypes::Home => {
            fs::read_to_string(&opts.hmcfg).expect("Failed to read file")
        }
        crate::npkgcmd::PackageTypes::System => {
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
        crate::npkgcmd::PackageTypes::Home => "home.packages",
        crate::npkgcmd::PackageTypes::System => "environment.systemPackages",
        _ => {
            println!("{}", "Unsupported package type".red());
            exit(1);
        }
    };

    let out = match action {
        Actions::Install => match pkwrite(pkgs, &f, Some(query)) {
            Ok(x) => x,
            Err(_) => exit(1),
        },
        Actions::Remove => match pkrm(pkgs, &f, Some(query)) {
            Ok(x) => x,
            Err(_) => exit(1),
        },
    };

    let outfile = match opts.output {
        Some(ref s) => s.to_string(),
        None => match opts.pkgmgr {
            crate::npkgcmd::PackageTypes::Home => opts.hmcfg.to_string(),
            crate::npkgcmd::PackageTypes::System => opts.syscfg.to_string(),
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
                println!(
                    "{} {}",
                    "Root permissions needed to modify".bright_yellow(),
                    &outfile.green()
                );

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
                        fs::remove_file(format!("{}/{}", cachedir, file))
                            .expect("Failed to remove file");
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
                            println!(
                                "{} {}",
                                "Root permissions needed to restore".bright_yellow(),
                                &outfile.green()
                            );

                            let file = outfile.split("/").collect::<Vec<&str>>().pop().unwrap();
                            let cachedir = format!("{}/.cache/npkg", env::var("HOME").unwrap());
                            fs::create_dir_all(&cachedir)
                                .expect("Failed to create cache directory");

                            match fs::write(format!("{}/{}", cachedir, file), &f) {
                                Ok(_) => {
                                    Command::new("sudo")
                                        .arg("cp")
                                        .arg(format!("{}/{}", cachedir, file))
                                        .arg(&outfile)
                                        .status()
                                        .expect(&format!(
                                            "{}",
                                            "Failed to execute process sudo cp".red()
                                        ));
                                    fs::remove_file(format!("{}/{}", cachedir, file))
                                        .expect("Failed to remove file");
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
        crate::npkgcmd::PackageTypes::Home => "home-manager".to_string(),
        crate::npkgcmd::PackageTypes::System => "nixos-rebuild".to_string(),
        _ => {
            println!("{}", "Unsupported package type".red());
            exit(1);
        }
    };

    match &opts.flake {
        None => match &opts.pkgmgr {
            crate::npkgcmd::PackageTypes::System => {
                println!("{}", "Need root access to rebuild system".bright_magenta());
                match systemswitch() {
                    Ok(()) => Ok(()),
                    Err(_) => Err(OperateError::CmdError),
                }
            }
            _ => match homeswitch() {
                Ok(()) => Ok(()),
                Err(_) => Err(OperateError::CmdError),
            },
        },
        Some(s) => {
            println!("Rebuilding with nix flakes");
            match &opts.pkgmgr {
                crate::npkgcmd::PackageTypes::System => {
                    println!("{}", "Need root access to rebuild system".bright_magenta());
                    match systemflakeswitch(s) {
                        Ok(()) => Ok(()),
                        Err(_) => Err(OperateError::CmdError),
                    }
                }
                _ => match homeflakeswitch(s) {
                    Ok(()) => Ok(()),
                    Err(_) => Err(OperateError::CmdError),
                },
            }
        }
    }
}
