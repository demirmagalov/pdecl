use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageConfig {
    pub version: u32,

    pub packages: Vec<String>,
}

impl PackageConfig {
    pub fn validate(&self, path: &Path) -> Result<()> {
        if self.version != CURRENT_VERSION {
            bail!(
                "{} uses unsupported config version {}. \
                 Supported version is {}.",
                path.display(),
                self.version,
                CURRENT_VERSION
            );
        }

        let mut seen = BTreeSet::new();

        for (index, package) in self.packages.iter().enumerate() {
            let package = package.trim();

            if package.is_empty() {
                bail!(
                    "{}: packages[{}] is empty",
                    path.display(),
                    index
                );
            }

            if package != self.packages[index] {
                bail!(
                    "{}: packages[{}] contains leading or trailing whitespace",
                    path.display(),
                    index
                );
            }

            if package.chars().any(char::is_whitespace) {
                bail!(
                    "{}: packages[{}] contains whitespace: {:?}",
                    path.display(),
                    index,
                    package
                );
            }

            if !seen.insert(package.to_string()) {
                bail!(
                    "{}: duplicate package {:?}",
                    path.display(),
                    package
                );
            }
        }

        Ok(())
    }

    pub fn package_set(&self) -> BTreeSet<String> {
        self.packages.iter().cloned().collect()
    }
}

pub fn load_config(path: &PathBuf) -> Result<PackageConfig> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("could not read {}", path.display()))?;

    let config: PackageConfig = toml::from_str(&contents)
        .with_context(|| format!("could not parse {}", path.display()))?;

    config.validate(path)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(packages: Vec<&str>) -> PackageConfig {
        PackageConfig {
            version: 1,
            packages: packages.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn accepts_valid_config() {
        let config = config(vec!["git", "neovim", "ripgrep"]);

        assert!(config.validate(Path::new("packages.toml")).is_ok());
    }

    #[test]
    fn rejects_empty_package_name() {
        let config = config(vec!["git", ""]);

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }

    #[test]
    fn rejects_whitespace() {
        let config = config(vec!["git", "hello world"]);

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }

    #[test]
    fn rejects_duplicate_package() {
        let config = config(vec!["git", "git"]);

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }

    #[test]
    fn rejects_unknown_version() {
        let mut config = config(vec!["git"]);
        config.version = 99;

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }
}
