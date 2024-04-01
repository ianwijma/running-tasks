use std::path::{Path, PathBuf};
use std::fs::canonicalize;

const DEFAULT_FILENAME: &str = "rask.yaml";

pub fn resolve_configuration_file(target: &String) -> Result<PathBuf, String> {
    let target_path = Path::new(target);

    let mut target = match canonicalize(target_path) {
        Ok(target) => target,
        Err(_) => return Err(format!("Target does not exists: {:?}", target))
    };

    if target.is_dir() {
        target.push(DEFAULT_FILENAME);
    }

    if !target.exists() {
        return Err(format!("Target does not exists: {:?}", target))
    }

    Ok(target)
}
