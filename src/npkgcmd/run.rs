use crate::npkgcmd::NpkgData;
use clap::{self, ArgGroup, Parser};
//use npkg::NpkgData;
use crate::npkgcmd::npkg;
use crate::npkgcmd::PackageTypes::*;
use owo_colors::*;
use std::process::exit;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("location")
        .args(&["system", "home", "env", "search"]),
))]
#[clap(group(
    ArgGroup::new("action")
        .args(&["install", "remove", "list", "search", "update"]),
))]
#[clap(group(
    ArgGroup::new("operations")
        .args(&["list", "packages"]),
))]
struct Args {
    /// Install a package
    #[clap(short, long)]
    install: bool,

    /// Remove a package
    #[clap(short, long)]
    remove: bool,

    /// List installed packages
    #[clap(short, long)]
    list: bool,

    /// Search for a package
    #[clap(short, long)]
    search: bool,

    /// Update packages
    #[clap(short, long)]
    update: bool,

    /// Use system 'configuration.nix'
    #[clap(short = 'S', long)]
    system: bool,

    /// Use home-manager 'home.nix'
    #[clap(short = 'H', long)]
    home: bool,

    /// Use nix environment 'nix-env'
    #[clap(short = 'E', long)]
    env: bool,

    /// Output modified configuration file to a specified location
    #[clap(short, long, conflicts_with_all = &["list", "search", "env", "update"])]
    output: Option<String>,

    /// Do not build any packages, only edit configuration file
    #[clap(short, long = "dry-run", conflicts_with_all = &["list", "search", "env", "update"])]
    dryrun: bool,

    /// Packages
    packages: Vec<String>,
}

fn printerror(msg: &str) {
    println!("{} {}", "error:".red(), msg);
}

fn pppackages(prepend: &str, packages: &Vec<String>) {
    println!("{} {}", prepend.green(), "Packages:".green());
    for package in packages {
        println!("  {}", package);
    }
}

fn pklst(opts: &NpkgData) -> Vec<String> {
    match opts.pkgmgr {
        System => match crate::npkgcmd::parse::syspkgs(opts.syscfg.to_string()) {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(crate::npkgcmd::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get system packages");
                exit(1);
            }
        },
        Home => match crate::npkgcmd::parse::hmpkgs(opts.hmcfg.to_string()) {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(crate::npkgcmd::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get home-manager packages");
                exit(1);
            }
        },
        Env => match crate::npkgcmd::parse::envpkgs() {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(crate::npkgcmd::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get nix environment packages");
                exit(1);
            }
        },
    }
}

fn pkinstall(mut opts: NpkgData) {
    match opts.pkgmgr {
        System => {
            opts.currpkgs = pklst(&opts);
            match crate::npkgcmd::operate::pkinstall(opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, do you have permissions in the directory \"{}\"?", f).as_str());
                    exit(1);
                }
            }
        }
        Home => {
            opts.currpkgs = pklst(&opts);
            match crate::npkgcmd::operate::pkinstall(opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Env => {
            opts.currpkgs = pklst(&opts);
            match crate::npkgcmd::operate::envinstall_check(opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not install packages");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(_)) => {
                    printerror("Could not write file");
                    exit(1);
                }
            }
        }
    }
}

fn pkremove(mut opts: NpkgData) {
    match opts.pkgmgr {
        System => {
            opts.currpkgs = pklst(&opts);
            match crate::npkgcmd::operate::pkremove(opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Home => {
            opts.currpkgs = pklst(&opts);
            match crate::npkgcmd::operate::pkremove(opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Env => {
            opts.currpkgs = pklst(&opts);
            match crate::npkgcmd::operate::envremove_check(opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not remove packages");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(_)) => {
                    printerror("Could not write file");
                    exit(1);
                }
            }
        }
    }
}

fn pkupdate(opts: &NpkgData) {
    match opts.pkgmgr {
        Home => {
            match crate::npkgcmd::operate::cfgswitch(&opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        System => {
            match crate::npkgcmd::operate::cfgswitch(&opts) {
                Ok(()) => {}
                Err(crate::npkgcmd::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(crate::npkgcmd::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            };
        }
        Env => {
            match npkg::envupdate() {
                Ok(()) => {}
                Err(npkg::ExecuteError::CmdError) => {
                    printerror("Could not update packages");
                    exit(1);
                }
                Err(npkg::ExecuteError::WriteError(_)) => {
                    printerror("Could not write file");
                    exit(1);
                }
            };
        }
    }
}

pub fn main() {
    let args = Args::parse();

    let hm = match std::process::Command::new("home-manager")
        .arg("--help")
        .output()
    {
        Ok(x) => {
            if x.status.success() {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    };

    let cfgdir = crate::npkgcmd::config::checkconfig();
    let (syscfg, hmcfg, flake) = crate::npkgcmd::config::readconfig(cfgdir);

    let mut opts = NpkgData {
        pkgmgr: Env,
        pkgs: args.packages,
        dryrun: args.dryrun || args.output.is_some(),
        output: args.output,
        syscfg: syscfg,
        hmcfg: hmcfg,
        flake: flake,
        currpkgs: vec![],
    };

    if args.install {
        if args.home {
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            opts.pkgmgr = Home;
            println!(
                "{} {}",
                "Installing package to".cyan(),
                "home".green().bold()
            );
            pkinstall(opts);
        } else if args.system {
            opts.pkgmgr = System;
            println!(
                "{} {}",
                "Installing package to".cyan(),
                "system".green().bold()
            );
            pkinstall(opts);
        } else {
            //Default env
            println!(
                "{} {}",
                "Installing package to".cyan(),
                "nix environment".green().bold()
            );
            pkinstall(opts);
        }
    } else if args.remove {
        if args.home {
            opts.pkgmgr = Home;
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            println!(
                "{} {}",
                "Removing package from".cyan(),
                "home".green().bold()
            );
            pkremove(opts);
        } else if args.system {
            opts.pkgmgr = System;
            println!(
                "{} {}",
                "Removing package from".cyan(),
                "system".green().bold()
            );
            pkremove(opts);
        } else {
            //Default env
            opts.pkgmgr = Env;
            println!(
                "{} {}",
                "Removing package from".cyan(),
                "nix environment".green().bold()
            );
            pkremove(opts);
        }
    } else if args.list {
        if args.home {
            //Add check for current packages
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            opts.pkgmgr = Home;
            let currpkgs = pklst(&opts);
            pppackages("Home Manager", &currpkgs);
        } else if args.system {
            opts.pkgmgr = System;
            let currpkgs = pklst(&opts);
            pppackages("System", &currpkgs);
        } else if args.env {
            opts.pkgmgr = Env;
            let currpkgs = pklst(&opts);
            pppackages("Nix Environment", &currpkgs);
        } else {
            //Default to all packages
            opts.pkgmgr = System;
            let syslst = pklst(&opts);
            opts.pkgmgr = Home;
            let homelst = if hm { pklst(&opts) } else { Vec::new() };
            opts.pkgmgr = Env;
            let envlst = pklst(&opts);
            pppackages("System", &syslst);
            if hm {
                pppackages("Home Manager", &homelst)
            }
            pppackages("Nix Environment", &envlst);
        }
    } else if args.search {
        // Get packages
        opts.pkgmgr = System;
        let syslst = pklst(&opts);
        opts.pkgmgr = Home;
        let homelst = if hm { pklst(&opts) } else { Vec::new() };
        opts.pkgmgr = Env;
        let envlst = pklst(&opts);

        // Search for packages
        let pkgdata = match crate::npkgcmd::search::search(&opts.pkgs) {
            Ok(x) => x,
            Err(_) => {
                printerror("Could not search for packages");
                exit(1);
            }
        };

        for pkg in pkgdata {
            let mut name = pkg.pname.to_string();

            let mut idx = vec![];
            let nl = name.to_lowercase();
            for st in &opts.pkgs {
                let sl = st.to_lowercase();
                let mut y = nl.match_indices(&sl).collect::<Vec<_>>();
                idx.append(&mut y);
            }
            idx.sort();
            let mut idx2: Vec<(usize, &str)> = vec![];
            for i in idx {
                if {
                    let mut b = true;
                    idx2.retain(|j| {
                        if j.0 == i.0 {
                            if i.1.len() > j.1.len() {
                                return false;
                            } else {
                                b = false;
                                return true;
                            }
                        }
                        return true;
                    });
                    b
                } {
                    idx2.push(i);
                }
            }
            let mut offset = 0;

            for (i, st) in idx2 {
                let j = i + offset;
                let n = name[j..j + st.len()].to_string();
                name.replace_range(j..j + st.len(), &n.green().to_string());
                offset += 10;
            }
            let mut outstr = format!("* {} ({})", name.bold(), pkg.version);
            if syslst.contains(&pkg.pname) {
                outstr += &format!(" ({})", "system".bright_red());
            }
            if homelst.contains(&pkg.pname) {
                outstr += &format!(" ({})", "home".bright_cyan());
            }
            if envlst.contains(&pkg.pname) {
                outstr += &format!(" ({})", "nix env".bright_yellow());
            }
            println!("{}", outstr);

            match pkg.description {
                Some(x) => {
                    let mut desc = x.trim().replace("\n", " ");
                    let mut idx = vec![];
                    let dl = desc.to_lowercase();
                    for st in &opts.pkgs {
                        let sl = st.to_lowercase();
                        let mut y = dl.match_indices(&sl).collect::<Vec<_>>();
                        idx.append(&mut y);
                    }
                    idx.sort();
                    let mut idx2: Vec<(usize, &str)> = vec![];
                    for i in idx {
                        if {
                            let mut b = true;
                            idx2.retain(|j| {
                                if j.0 == i.0 {
                                    if i.1.len() > j.1.len() {
                                        return false;
                                    } else {
                                        b = false;
                                        return true;
                                    }
                                }
                                return true;
                            });
                            b
                        } {
                            idx2.push(i);
                        }
                    }
                    let mut offset = 0;
                    for (i, st) in idx2 {
                        let j = i + offset;
                        let d = desc[j..j + st.len()].to_string();
                        desc.replace_range(j..j + st.len(), &d.green().to_string());
                        offset += 10;
                    }
                    println!("  {}", desc)
                }
                None => {}
            }
            println!();
        }
    } else if args.update {
        if args.home {
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            opts.pkgmgr = Home;
            println!(
                "{} {}",
                "Updating packages in".cyan(),
                "home".green().bold()
            );
            crate::npkgcmd::operate::chnupdate(&opts);
            pkupdate(&opts);
        } else if args.system {
            opts.pkgmgr = System;
            println!(
                "{} {}",
                "Updating packages in".cyan(),
                "system".green().bold()
            );
            crate::npkgcmd::operate::chnupdate(&opts);
            pkupdate(&opts);
        } else if args.env {
            //Default env
            opts.pkgmgr = Env;
            println!(
                "{} {}",
                "Updating packages in".cyan(),
                "nix environment".green().bold()
            );
            crate::npkgcmd::operate::chnupdate(&opts);
            pkupdate(&opts);
        } else {
            crate::npkgcmd::operate::chnupdate(&opts);
            opts.pkgmgr = System;
            println!(
                "{} {}",
                "Updating packages in".cyan(),
                "system".green().bold()
            );
            pkupdate(&opts);
            if hm {
                opts.pkgmgr = Home;
                println!(
                    "{} {}",
                    "Updating packages in".cyan(),
                    "home".green().bold()
                );
                pkupdate(&opts);
            }
            opts.pkgmgr = Env;
            println!(
                "{} {}",
                "Updating packages in".cyan(),
                "nix environment".green().bold()
            );
            pkupdate(&opts);
        }
    } else {
        printerror("no operation specified");
        println!("Try 'npkg --help' for more information.");
        exit(-1);
    }
}
