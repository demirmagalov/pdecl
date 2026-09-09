use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDiff {
    pub install: BTreeSet<String>,
    pub remove: BTreeSet<String>,
}

impl PackageDiff {
    pub fn between(
        desired: &BTreeSet<String>,
        actual: &BTreeSet<String>,
    ) -> Self {
        let install = desired
            .difference(actual)
            .cloned()
            .collect();

        let remove = actual
            .difference(desired)
            .cloned()
            .collect();

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
            output.push_str(&format!("  + {package}\n"));
        }

        output.push('\n');
    }

    if !diff.remove.is_empty() {
        output.push_str("Packages to remove:\n");

        for package in &diff.remove {
            output.push_str(&format!("  - {package}\n"));
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
        if diff.change_count() == 1 {
            ""
        } else {
            "s"
        }
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

        let diff = PackageDiff::between(&desired, &actual);

        assert_eq!(diff.install, set(&["ripgrep"]));
        assert_eq!(diff.remove, set(&["nano"]));
    }

    #[test]
    fn identical_sets_produce_empty_diff() {
        let desired = set(&["git", "neovim"]);
        let actual = set(&["git", "neovim"]);

        let diff = PackageDiff::between(&desired, &actual);

        assert!(diff.is_empty());
    }

    #[test]
    fn empty_actual_means_everything_is_installed() {
        let desired = set(&["git", "neovim"]);
        let actual = set(&[]);

        let diff = PackageDiff::between(&desired, &actual);

        assert_eq!(diff.install, set(&["git", "neovim"]));
        assert!(diff.remove.is_empty());
    }

    #[test]
    fn empty_desired_means_everything_is_removed() {
        let desired = set(&[]);
        let actual = set(&["git", "neovim"]);

        let diff = PackageDiff::between(&desired, &actual);

        assert!(diff.install.is_empty());
        assert_eq!(diff.remove, set(&["git", "neovim"]));
    }
}
