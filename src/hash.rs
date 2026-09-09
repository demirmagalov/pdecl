use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub fn hash_directory(root: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;

    files.sort();

    let mut hasher = Sha256::new();

    for relative in files {
        let full = root.join(&relative);

        let content =
            fs::read(&full).with_context(|| format!("could not read {}", full.display()))?;

        let path_bytes = relative.to_string_lossy().as_bytes().to_vec();

        hasher.update((path_bytes.len() as u64).to_le_bytes());
        hasher.update(path_bytes);

        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_files(root: &Path, current: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("could not read {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            collect_files(root, &path, output)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .with_context(|| {
                    format!(
                        "could not make {} relative to {}",
                        path.display(),
                        root.display()
                    )
                })?
                .to_path_buf();

            output.push(relative);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_directories_have_identical_hashes() {
        let temp = tempfile::tempdir().unwrap();

        fs::write(temp.path().join("PKGBUILD"), b"pkgname=test").unwrap();

        let first = hash_directory(temp.path()).unwrap();
        let second = hash_directory(temp.path()).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn changing_a_file_changes_the_hash() {
        let temp = tempfile::tempdir().unwrap();

        let path = temp.path().join("PKGBUILD");

        fs::write(&path, b"pkgname=test").unwrap();
        let first = hash_directory(temp.path()).unwrap();

        fs::write(&path, b"pkgname=changed").unwrap();
        let second = hash_directory(temp.path()).unwrap();

        assert_ne!(first, second);
    }
}
