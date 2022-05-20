use std::process::{exit, Command};

pub enum ExecuteError {
    /// Thrown when running a command fails.
    CmdError,
    /// Thrown when writing to a file fails.
    /// The path to the file is included.
    WriteError(String),
}

/// Installs packages using `nix-env -iA nixos.<pkg>`
///
/// Packages must be in the `nixos` channel
pub fn envinstall(pkgs: Vec<String>) -> Result<(), ExecuteError> {
    let mut prefixpkgs = vec![];
    for p in &pkgs {
        prefixpkgs.push(format!("nixos.{}", p));
    }
    match Command::new("nix-env").arg("-iA").args(prefixpkgs).status() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Uninstalls packages using `nix-env -e <pkg>`
///
/// Uninstalling is based on the package name rather than the package attribute used during install
pub fn envremove(pkgs: Vec<String>) -> Result<(), ExecuteError> {
    match Command::new("nix-env").arg("-e").args(pkgs).status() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Updates packages using `nix-env -u *`
pub fn envupdate() -> Result<(), ExecuteError> {
    match Command::new("nix-env").arg("-u").arg("*").status() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Adds packages to a nix configuration file
///
/// Package specified are added to the configuration file text in `f`.
/// The configuration text with the added package is returned.
/// By default the field modified is `environment.systemPackages`, but can be changed if `query` is specified.
pub fn pkwrite(
    mut pkgs: Vec<String>,
    f: &str,
    query: Option<&str>,
) -> Result<String, ExecuteError> {
    let q;
    (pkgs, q) = pkwith(pkgs, f, query);
    let out = match nix_editor::write::addtoarr(f, &q, pkgs) {
        Ok(x) => x,
        Err(_) => exit(1),
    };

    Ok(out)
}

/// Removes packages from a nix configuration file
///
/// Package specified are removed from the configuration text in `f`.
/// The configuration text with the removed package is returned.
/// By default the field modified is `environment.systemPackages`, but can be changed if `query` is specified.
pub fn pkrm(mut pkgs: Vec<String>, f: &str, query: Option<&str>) -> Result<String, ExecuteError> {
    let q;
    (pkgs, q) = pkwith(pkgs, f, query);
    let out = match nix_editor::write::rmarr(f, &q, pkgs) {
        Ok(x) => x,
        Err(_) => exit(1),
    };

    Ok(out)
}

fn pkwith(mut pkgs: Vec<String>, f: &str, query: Option<&str>) -> (Vec<String>, String) {
    let q = query.unwrap_or("environment.systemPackages");

    if let Ok(s) = nix_editor::read::getwithvalue(f, q) {
        if !s.contains(&"pkgs".to_string()) {
            pkgs = pkgs
                .into_iter()
                .map(|x| format!("pkgs.{}", x))
                .collect::<Vec<String>>();
        }
    }

    (pkgs, q.to_string())
}

/// Calls `nixos-rebuild switch`
pub fn systemswitch() -> Result<(), ExecuteError> {
    match Command::new("nixos-rebuild").arg("switch").status() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Calls `nixos-rebuild switch` with the `--flake` flag
///
/// The input `flakepath` is the path to the flake file with any arguments.
/// Eg `/etc/nixos#user`.
pub fn systemflakeswitch(flakepath: &str) -> Result<(), ExecuteError> {
    let status = Command::new("nixos-rebuild")
        .arg("switch")
        .arg("--flake")
        .arg(flakepath)
        .arg("--use-remote-sudo")
        .status();
    match status {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Calls `home-manager switch`
pub fn homeswitch() -> Result<(), ExecuteError> {
    let status = Command::new("home-manager").arg("switch").status();
    match status {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Calls `home-manager switch` with the `--flake` flag
///
/// The input `flakepath` is the path to the flake file with any arguments.
/// Eg `/home/user/nix#user`.
pub fn homeflakeswitch(flakepath: &str) -> Result<(), ExecuteError> {
    let status = Command::new("home-manager")
        .arg("switch")
        .arg("--flake")
        .arg(flakepath)
        .status();
    match status {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Calls `nix-channel --update`
pub fn updatechannel() -> Result<(), ExecuteError> {
    match Command::new("nix-channel").arg("--update").status() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}

/// Calls `nix flake update` on the specified flake
///
/// The input `flake` is the path to the flake file.
pub fn updateflake(flake: &str) -> Result<(), ExecuteError> {
    match Command::new("nix")
        .arg("flake")
        .arg("update")
        .arg(flake.split('#').collect::<Vec<&str>>()[0])
        .status()
    {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecuteError::CmdError),
    }
}
