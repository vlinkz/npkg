use clap::{self, ArgGroup, Parser};
use npkg::NpkgData;
use npkg::PackageTypes::*;
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
        System => match npkg::parse::syspkgs(opts.syscfg.to_string()) {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(npkg::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get system packages");
                exit(1);
            }
        },
        Home => match npkg::parse::hmpkgs(opts.hmcfg.to_string()) {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(npkg::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get home-manager packages");
                exit(1);
            }
        },
        Env => match npkg::parse::envpkgs() {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(npkg::parse::ParseError::EmptyPkgs) => {
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
            match npkg::operate::pkinstall(opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Home => {
            opts.currpkgs = pklst(&opts);
            match npkg::operate::pkinstall(opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Env => {
            opts.currpkgs = pklst(&opts);
            match npkg::operate::envinstall(opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not install packages");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(_)) => {
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
            match npkg::operate::pkremove(opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Home => {
            opts.currpkgs = pklst(&opts);
            match npkg::operate::pkremove(opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        }
        Env => {
            opts.currpkgs = pklst(&opts);
            match npkg::operate::envremove(opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not remove packages");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(_)) => {
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
            match npkg::operate::cfgswitch(&opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            }
        },
        System => {
            match npkg::operate::cfgswitch(&opts) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(f)) => {
                    printerror(format!("Could not write to configuration file, does the directory \"{}\" exist?", f).as_str());
                    exit(1);
                }
            };
        },
        Env => {
            match npkg::operate::envupdate() {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not update packages");
                    exit(1);
                }
                Err(npkg::operate::OperateError::WriteError(_)) => {
                    printerror("Could not write file");
                    exit(1);
                }
            };
        },
    }
}

fn main() {
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

    npkg::config::checkconfig();
    let (syscfg, hmcfg, flake) = npkg::config::readconfig();

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
            println!("{} {}", "Installing package to".cyan(), "home".green().bold());
            pkinstall(opts);
        } else if args.system {
            opts.pkgmgr = System;
            println!("{} {}", "Installing package to".cyan(), "system".green().bold());
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
            println!("{} {}", "Removing package from".cyan(), "home".green().bold());
            pkremove(opts);
        } else if args.system {
            opts.pkgmgr = System;
            println!("{} {}", "Removing package from".cyan(), "system".green().bold());
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
            let currpkgs = pklst(&opts);
            pppackages("Home Manager", &currpkgs);
        } else if args.system {
            let currpkgs = pklst(&opts);
            pppackages("System", &currpkgs);
        } else if args.env {
            let currpkgs = pklst(&opts);
            pppackages("Nix Environment", &currpkgs);
        } else {
            //Default to all packages
            opts.pkgmgr = System;
            let syslst = pklst(&opts);
            opts.pkgmgr = Home;
            let homelst = if hm {pklst(&opts)} else {Vec::new()};
            opts.pkgmgr = Env;
            let envlst = pklst(&opts);
            pppackages("System", &syslst);
            if hm { pppackages("Home Manager", &homelst) }
            pppackages("Nix Environment", &envlst);
        }
    } else if args.search {
        let pkgdata = match npkg::search::search(&opts.pkgs) {
            Ok(x) => x,
            Err(_) => {
                printerror("Could not search for packages");
                exit(1);
            }
        };

        for pkg in pkgdata {
            let mut name = pkg.pname;
            for st in &opts.pkgs {
                let nl = name.to_lowercase();
                let sl = st.to_lowercase();
                let y = nl.match_indices(&sl);
                let mut offset = 0;
                for (i, _) in y {
                    let j = i + offset;
                    let n = name[j..j + st.len()].to_string();
                    name.replace_range(j..j + st.len(), &n.green().to_string());
                    offset += 10;
                }

                name = name.replace(st.as_str(), &st.green().to_string());
            }
            println!("* {} ({})", name.bold(), pkg.version);
            match pkg.description {
                Some(x) => {
                    let mut desc = x.trim().replace("\n", " ");
                    for st in &opts.pkgs {
                        let dl = desc.to_lowercase();
                        let sl = st.to_lowercase();
                        let y = dl.match_indices(&sl);
                        let mut offset = 0;
                        for (i, _) in y {
                            let j = i + offset;
                            let d = desc[j..j + st.len()].to_string();
                            desc.replace_range(j..j + st.len(), &d.green().to_string());
                            offset += 10;
                        }
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
            println!("{} {}", "Updating packages in".cyan(), "home".green().bold());
            npkg::operate::chnupdate(&opts);
            pkupdate(&opts);
        } else if args.system {
            opts.pkgmgr = System;
            println!("{} {}", "Updating packages in".cyan(), "system".green().bold());
            npkg::operate::chnupdate(&opts);
            pkupdate(&opts);
        } else if args.env {
            //Default env
            opts.pkgmgr = Env;
            println!(
                "{} {}",
                "Updating packages in".cyan(),
                "nix environment".green().bold()
            );
            npkg::operate::chnupdate(&opts);
            pkupdate(&opts);
        } else {
            npkg::operate::chnupdate(&opts);
            opts.pkgmgr = System;
            println!("{} {}", "Updating packages in".cyan(), "system".green().bold());
            pkupdate(&opts);
            opts.pkgmgr = Home;
            println!("{} {}", "Updating packages in".cyan(), "home".green().bold());
            pkupdate(&opts);
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
