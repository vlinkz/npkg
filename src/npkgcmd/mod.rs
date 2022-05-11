pub mod parse;
pub mod operate;
pub mod search;
pub mod config;
pub mod run;
use npkg;

pub struct PkgData {
    pub pname: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(Debug)]

pub enum PackageTypes {
    System,
    Home,
    Env,
}

#[derive(Debug)]
pub struct NpkgData {
    pub pkgmgr: PackageTypes,
    pub pkgs: Vec<String>,
    pub output: Option<String>,
    pub syscfg: String,
    pub hmcfg: String,
    pub dryrun: bool,
    pub flake: Option<String>,
    pub currpkgs: Vec<String>,
}