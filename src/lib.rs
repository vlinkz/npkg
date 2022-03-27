pub mod parse;
pub mod operate;
pub mod search;

pub struct PkgData {
    pub pname: String,
    pub description: Option<String>,
    pub version: String,
}