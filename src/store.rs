use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};

pub fn default_store() -> Result<PathBuf> {
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg_data_home).join("pdecl"));
    }

    let home = env::var("HOME").context("HOME is not set")?;

    Ok(PathBuf::from(home).join(".local/share/pdecl"))
}

pub fn aur_store() -> Result<PathBuf> {
    Ok(default_store()?.join("aur"))
}

pub fn ensure_aur_store() -> Result<PathBuf> {
    let path = aur_store()?;

    fs::create_dir_all(&path).with_context(|| format!("could not create {}", path.display()))?;

    Ok(path)
}
