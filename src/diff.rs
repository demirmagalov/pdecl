use std::collections::BTreeSet;

use crate::config::PackageSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageChange {
    pub name: String,
    pub source: PackageSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDiff {
    pub install: Vec<PackageChange>,
    pub remove: Vec<PackageChange>,
}

impl PackageDiff {
    pub fn between(
        desired: &BTreeSet<String>,
        desired_sources: impl Fn(&str) -> PackageSource,
        actual: &BTreeSet<String>,
        foreign: &BTreeSet<String>,
    ) -> Self {
        let mut install = Vec::new();
        let mut remove = Vec::new();

        for package in desired.difference(actual) {
            install.push(PackageChange {
                name: package.clone(),
                source: desired_sources(package),
            });
        }

        for package in actual.difference(desired) {
            remove.push(PackageChange {
                name: package.clone(),
                source: if foreign.contains(package) {
                    PackageSource::Aur
                } else {
                    PackageSource::Repo
                },
            });
        }

        Self { install, remove }
    }

    pub fn is_empty(&self) -> bool {
        self.install.is_empty() && self.remove.is_empty()
    }

    pub fn change_count(&self) -> usize {
        self.install.len() + self.remove.len()
    }
}

pub fn format_diff(diff: &PackageDiff) -> String {
    let mut output = String::new();

    if !diff.install.is_empty() {
        output.push_str("Packages to install:\n");

        for package in &diff.install {
            let source = match package.source {
                PackageSource::Repo => "repo",
                PackageSource::Aur => "aur",
            };

            output.push_str(&format!("  + {} ({source})\n", package.name));
        }

        output.push('\n');
    }

    if !diff.remove.is_empty() {
        output.push_str("Packages to remove:\n");

        for package in &diff.remove {
            let source = match package.source {
                PackageSource::Repo => "repo",
                PackageSource::Aur => "aur",
            };

            output.push_str(&format!("  - {} ({source})\n", package.name));
        }

        output.push('\n');
    }

    if diff.is_empty() {
        output.push_str("No changes.\n");
        return output;
    }

    output.push_str(&format!(
        "Summary: {} package{} to change.\n",
        diff.change_count(),
        if diff.change_count() == 1 { "" } else { "s" }
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn calculates_install_and_remove() {
        let desired = set(&["git", "neovim", "ripgrep"]);
        let actual = set(&["git", "neovim", "nano"]);
        let foreign = set(&[]);

        let diff = PackageDiff::between(&desired, |_| PackageSource::Repo, &actual, &foreign);

        assert_eq!(diff.install.len(), 1);
        assert_eq!(diff.install[0].name, "ripgrep");
        assert_eq!(diff.install[0].source, PackageSource::Repo);

        assert_eq!(diff.remove.len(), 1);
        assert_eq!(diff.remove[0].name, "nano");
        assert_eq!(diff.remove[0].source, PackageSource::Repo);
    }

    #[test]
    fn detects_aur_install() {
        let desired = set(&["git", "yay-bin"]);
        let actual = set(&["git"]);
        let foreign = set(&[]);

        let diff = PackageDiff::between(
            &desired,
            |name| {
                if name == "yay-bin" {
                    PackageSource::Aur
                } else {
                    PackageSource::Repo
                }
            },
            &actual,
            &foreign,
        );

        assert_eq!(diff.install.len(), 1);
        assert_eq!(diff.install[0].name, "yay-bin");
        assert_eq!(diff.install[0].source, PackageSource::Aur);
    }

    #[test]
    fn detects_foreign_removal() {
        let desired = set(&["git"]);
        let actual = set(&["git", "yay-bin"]);
        let foreign = set(&["yay-bin"]);

        let diff = PackageDiff::between(&desired, |_| PackageSource::Repo, &actual, &foreign);

        assert_eq!(diff.remove.len(), 1);
        assert_eq!(diff.remove[0].name, "yay-bin");
        assert_eq!(diff.remove[0].source, PackageSource::Aur);
    }

    #[test]
    fn identical_sets_produce_empty_diff() {
        let desired = set(&["git", "neovim"]);
        let actual = set(&["git", "neovim"]);
        let foreign = set(&[]);

        let diff = PackageDiff::between(&desired, |_| PackageSource::Repo, &actual, &foreign);

        assert!(diff.is_empty());
    }
}
