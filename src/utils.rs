use std::path::{Path, PathBuf};
use std::fs;
use super::Result;

const SUFFIX: &str = ".log";

pub fn ls_logs(path: &Path) -> Vec<u32> {
    let mut v = path.read_dir().unwrap().into_iter()
        .map(|p| p.unwrap().file_name().to_str().unwrap().to_string())
        .filter(|name| name.ends_with(".log"))
        .filter_map(|name| name.split_at(name.len() - 4).0.parse::<u32>().ok())
        .collect::<Vec<u32>>();
    v.sort();
    v
}

pub fn format_path(path: &Path, id: u32) -> PathBuf {
    path.join(format!("{}{}", id, SUFFIX))
}

pub fn del_file(path: PathBuf) -> Result<()> {
    Ok(fs::remove_file(path)?)
}