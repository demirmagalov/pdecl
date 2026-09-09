use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const CURRENT_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageSource {
    Repo,
    Aur,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub source: PackageSource,
}

#[derive(Debug, Clone)]
pub struct PackageConfig {
    pub version: u32,
    pub packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct V1Config {
    version: u32,
    packages: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct V2Config {
    version: u32,
    #[serde(rename = "package")]
    packages: Vec<V2Package>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct V2Package {
    name: String,
    source: PackageSource,
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
            let name = package.name.trim();

            if name.is_empty() {
                bail!("{}: package[{}].name is empty", path.display(), index);
            }

            if name != package.name {
                bail!(
                    "{}: package[{}].name contains leading or trailing whitespace",
                    path.display(),
                    index
                );
            }

            if name.chars().any(char::is_whitespace) {
                bail!(
                    "{}: package[{}].name contains whitespace: {:?}",
                    path.display(),
                    index,
                    name
                );
            }

            if !seen.insert(name.to_string()) {
                bail!("{}: duplicate package {:?}", path.display(), name);
            }
        }

        Ok(())
    }

    pub fn package_set(&self) -> BTreeSet<String> {
        self.packages
            .iter()
            .map(|package| package.name.clone())
            .collect()
    }

    pub fn aur_packages(&self) -> BTreeSet<String> {
        self.packages
            .iter()
            .filter(|package| package.source == PackageSource::Aur)
            .map(|package| package.name.clone())
            .collect()
    }

    pub fn package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|package| package.name == name)
    }
}

pub fn load_config(path: &PathBuf) -> Result<PackageConfig> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;

    let value: toml::Value =
        toml::from_str(&contents).with_context(|| format!("could not parse {}", path.display()))?;

    let version = value
        .get("version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| anyhow::anyhow!("{} is missing a valid version", path.display()))?;

    let config = match version {
        1 => {
            let old: V1Config = value
                .try_into()
                .with_context(|| format!("could not parse {}", path.display()))?;

            if old.version != 1 {
                bail!("{} has invalid version {}", path.display(), old.version);
            }

            PackageConfig {
                version: CURRENT_VERSION,
                packages: old
                    .packages
                    .into_iter()
                    .map(|name| Package {
                        name,
                        source: PackageSource::Repo,
                    })
                    .collect(),
            }
        }

        2 => {
            let new: V2Config = value
                .try_into()
                .with_context(|| format!("could not parse {}", path.display()))?;

            PackageConfig {
                version: new.version,
                packages: new
                    .packages
                    .into_iter()
                    .map(|package| Package {
                        name: package.name,
                        source: package.source,
                    })
                    .collect(),
            }
        }

        other => {
            bail!(
                "{} uses unsupported config version {}. \
                 Supported versions are 1 and 2.",
                path.display(),
                other
            );
        }
    };

    config.validate(path)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_config(packages: Vec<&str>) -> PackageConfig {
        PackageConfig {
            version: 2,
            packages: packages
                .into_iter()
                .map(|name| Package {
                    name: name.to_string(),
                    source: PackageSource::Repo,
                })
                .collect(),
        }
    }

    #[test]
    fn accepts_valid_config() {
        let config = repo_config(vec!["git", "neovim", "ripgrep"]);

        assert!(config.validate(Path::new("packages.toml")).is_ok());
    }

    #[test]
    fn accepts_aur_package() {
        let config = PackageConfig {
            version: 2,
            packages: vec![Package {
                name: "neovim-git".to_string(),
                source: PackageSource::Aur,
            }],
        };

        assert!(config.validate(Path::new("packages.toml")).is_ok());
        assert_eq!(
            config.aur_packages(),
            BTreeSet::from(["neovim-git".to_string()])
        );
    }

    #[test]
    fn rejects_empty_package_name() {
        let config = repo_config(vec!["git", ""]);

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }

    #[test]
    fn rejects_whitespace() {
        let config = repo_config(vec!["git", "hello world"]);

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }

    #[test]
    fn rejects_duplicate_package() {
        let config = repo_config(vec!["git", "git"]);

        assert!(config.validate(Path::new("packages.toml")).is_err());
    }

    #[test]
    fn rejects_unknown_version() {
        let config = repo_config(vec!["git"]);

        let path = Path::new("packages.toml");
        let contents = r#"
version = 99

[[package]]
name = "git"
source = "repo"
"#;

        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), contents).unwrap();

        assert!(load_config(&temp.path().to_path_buf()).is_err());

        assert!(config.validate(path).is_ok());
    }
}
