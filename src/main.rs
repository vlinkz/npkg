use clap::{self, ArgGroup, Parser};
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
        .args(&["install", "remove", "list", "search"]),
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

    /// Use system '/etc/nixos/configuration.nix'
    #[clap(short = 'S', long)]
    system: bool,

    /// Use home-manager '~/.config/nixpkgs/home.nix'
    #[clap(short = 'H', long)]
    home: bool,

    /// Use nix environment 'nix-env'
    #[clap(short = 'E', long)]
    env: bool,

    /// Packages
    packages: Vec<String>,
}

enum PackageTypes {
    System,
    Home,
    Env,
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

fn pklst(pkmgr: PackageTypes) -> Vec<String> {
    match pkmgr {
        PackageTypes::System => match npkg::parse::syspkgs() {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(npkg::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get system packages");
                exit(1);
            }
        },
        PackageTypes::Home => match npkg::parse::hmpkgs() {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(npkg::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get home-manager packages");
                exit(1);
            }
        },
        PackageTypes::Env => match npkg::parse::envpkgs() {
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

fn pkinstall(pkmgr: PackageTypes, pkgs: Vec<String>) {
    match pkmgr {
        PackageTypes::System => {
            let currpkgs = pklst(PackageTypes::System);
            match npkg::operate::sysinstall(pkgs, currpkgs) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
            }
        }
        PackageTypes::Home => {
            let currpkgs = pklst(PackageTypes::Home);
            match npkg::operate::hminstall(pkgs, currpkgs) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
            }
        }
        PackageTypes::Env => {
            let currpkgs = pklst(PackageTypes::Env);
            match npkg::operate::envinstall(pkgs, currpkgs) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not install packages");
                    exit(1);
                }
            }
        }
    }
}

fn pkremove(pkmgr: PackageTypes, pkgs: Vec<String>) {
    match pkmgr {
        PackageTypes::System => {
            let currpkgs = pklst(PackageTypes::System);
            match npkg::operate::sysremove(pkgs, currpkgs) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
            }
        }
        PackageTypes::Home => {
            let currpkgs = pklst(PackageTypes::Home);
            match npkg::operate::hmremove(pkgs, currpkgs) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not rebuild configuration");
                    exit(1);
                }
            }
        }
        PackageTypes::Env => {
            let currpkgs = pklst(PackageTypes::Env);
            match npkg::operate::envremove(pkgs, currpkgs) {
                Ok(()) => {}
                Err(npkg::operate::OperateError::CmdError) => {
                    printerror("Could not remove packages");
                    exit(1);
                }
            }
        }
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

    if args.install {
        if args.home {
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            println!("{} {}", "Installing package to ".cyan(), "home".green());
            pkinstall(PackageTypes::Home, args.packages);
        } else if args.system {
            println!("{} {}", "Installing package to ".cyan(), "system".green());
            pkinstall(PackageTypes::System, args.packages);
        } else {
            //Default env
            println!(
                "{} {}",
                "Installing package to ".cyan(),
                "nix environment".green()
            );
            pkinstall(PackageTypes::Env, args.packages);
        }
    } else if args.remove {
        if args.home {
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            println!("{} {}", "Removing package from ".cyan(), "home".green());
            pkremove(PackageTypes::Home, args.packages);
        } else if args.system {
            println!("{} {}", "Removing package from ".cyan(), "system".green());
            pkremove(PackageTypes::System, args.packages);
        } else {
            //Default env
            println!(
                "{} {}",
                "Removing package from ".cyan(),
                "nix environment".green()
            );
            pkremove(PackageTypes::Env, args.packages);
        }
    } else if args.list {
        if args.home {
            //Add check for current packages
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            let currpkgs = pklst(PackageTypes::Home);
            pppackages("Home Manager", &currpkgs);
        } else if args.system {
            let currpkgs = pklst(PackageTypes::System);
            pppackages("System", &currpkgs);
        } else if args.env {
            let currpkgs = pklst(PackageTypes::Env);
            pppackages("Nix Environment", &currpkgs);
        } else {
            //Default to all packages
            pppackages("System", &pklst(PackageTypes::System));
            pppackages("Home Manager", &pklst(PackageTypes::Home));
            pppackages("Nix Environment", &pklst(PackageTypes::Env));
        }
    } else if args.search {
        let pkgdata = match npkg::search::search(&args.packages) {
            Ok(x) => x,
            Err(_) => {
                printerror("Could not search for packages");
                exit(1);
            }
        };

        for pkg in pkgdata {
            let mut name = pkg.pname;
            for st in &args.packages {
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
                    for st in &args.packages {
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
    } else {
        printerror("no operation specified");
        println!("Try 'npkg --help' for more information.");
        exit(-1);
    }
}
