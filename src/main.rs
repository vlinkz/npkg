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
    #[clap(short, long, conflicts_with_all = &["list", "search", "env"])]
    output: Option<String>,

    /// Do not build any packages, only edit configuration file
    #[clap(short, long = "dry-run", conflicts_with_all = &["list", "search", "env"])]
    dryrun: bool,

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

fn pklst(pkmgr: PackageTypes, cfg: &Option<String>) -> Vec<String> {
    match pkmgr {
        PackageTypes::System => match npkg::parse::syspkgs(cfg.to_owned().unwrap()) {
            Ok(mut x) => {
                x.sort();
                x
            }
            Err(npkg::parse::ParseError::EmptyPkgs) => {
                printerror("Failed to get system packages");
                exit(1);
            }
        },
        PackageTypes::Home => match npkg::parse::hmpkgs(cfg.to_owned().unwrap()) {
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

fn pkinstall(pkmgr: PackageTypes, pkgs: Vec<String>, output: Option<String>, cfg: Option<String>, dryrun: bool, flake: Option<String>) {
    match pkmgr {
        PackageTypes::System => {
            let currpkgs = pklst(PackageTypes::System, &cfg);
            match npkg::operate::sysinstall(cfg.unwrap(), pkgs, currpkgs, output, !dryrun, flake) {
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
        PackageTypes::Home => {
            let currpkgs = pklst(PackageTypes::Home, &cfg);
            match npkg::operate::hminstall(cfg.unwrap(), pkgs, currpkgs, output, !dryrun, flake) {
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
        PackageTypes::Env => {
            let currpkgs = pklst(PackageTypes::Env, &cfg);
            match npkg::operate::envinstall(pkgs, currpkgs) {
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

fn pkremove(pkmgr: PackageTypes, pkgs: Vec<String>, output: Option<String>, cfg: Option<String>, dryrun: bool, flake: Option<String>) {
    match pkmgr {
        PackageTypes::System => {
            let currpkgs = pklst(PackageTypes::System, &cfg);
            match npkg::operate::sysremove(cfg.unwrap(), pkgs, currpkgs, output, !dryrun, flake) {
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
        PackageTypes::Home => {
            let currpkgs = pklst(PackageTypes::Home, &cfg);
            match npkg::operate::hmremove(cfg.unwrap(), pkgs, currpkgs, output, !dryrun, flake) {
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
        PackageTypes::Env => {
            let currpkgs = pklst(PackageTypes::Env, &cfg);
            match npkg::operate::envremove(pkgs, currpkgs) {
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

    if args.install {
        if args.home {
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            println!("{} {}", "Installing package to".cyan(), "home".green().bold());
            let dry = args.dryrun || args.output.is_some();
            pkinstall(PackageTypes::Home, args.packages, args.output, Some(hmcfg), dry, flake);
        } else if args.system {
            println!("{} {}", "Installing package to".cyan(), "system".green().bold());
            let dry = args.dryrun || args.output.is_some();
            pkinstall(PackageTypes::System, args.packages, args.output, Some(syscfg), dry, flake);
        } else {
            //Default env
            println!(
                "{} {}",
                "Installing package to".cyan(),
                "nix environment".green().bold()
            );
            pkinstall(PackageTypes::Env, args.packages, None, None,false, flake);
        }
    } else if args.remove {
        if args.home {
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            println!("{} {}", "Removing package from".cyan(), "home".green().bold());
            let dry = args.dryrun || args.output.is_some();
            pkremove(PackageTypes::Home, args.packages, args.output, Some(hmcfg), dry, flake);
        } else if args.system {
            println!("{} {}", "Removing package from".cyan(), "system".green().bold());
            let dry = args.dryrun || args.output.is_some();
            pkremove(PackageTypes::System, args.packages, args.output, Some(syscfg), dry, flake);
        } else {
            //Default env
            println!(
                "{} {}",
                "Removing package from".cyan(),
                "nix environment".green().bold()
            );
            pkremove(PackageTypes::Env, args.packages, None, None, false, flake);
        }
    } else if args.list {
        if args.home {
            //Add check for current packages
            if !hm {
                printerror("home-manager is not installed");
                exit(1);
            }
            let currpkgs = pklst(PackageTypes::Home, &Some(hmcfg));
            pppackages("Home Manager", &currpkgs);
        } else if args.system {
            let currpkgs = pklst(PackageTypes::System, &Some(syscfg));
            pppackages("System", &currpkgs);
        } else if args.env {
            let currpkgs = pklst(PackageTypes::Env, &None);
            pppackages("Nix Environment", &currpkgs);
        } else {
            //Default to all packages
            let syslst = pklst(PackageTypes::System, &Some(syscfg));
            let homelst = if hm {pklst(PackageTypes::Home, &Some(hmcfg))} else {Vec::new()};
            let envlst = pklst(PackageTypes::Env, &None);
            pppackages("System", &syslst);
            if hm { pppackages("Home Manager", &homelst) }
            pppackages("Nix Environment", &envlst);
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
