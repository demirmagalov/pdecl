use std::{
    collections::BTreeSet,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, anyhow};

#[derive(Debug, Clone, Copy)]
pub struct Pacman;

impl Pacman {
    pub fn new() -> Self {
        Self
    }

    pub fn explicit_packages(&self) -> Result<BTreeSet<String>> {
        let output = Command::new("pacman")
            .args(["-Qqe"])
            .output()
            .context("could not execute pacman")?;

        if !output.status.success() {
            return Err(anyhow!(
                "pacman -Qqe exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let stdout = String::from_utf8(output.stdout).context("pacman produced invalid UTF-8")?;

        let mut packages = BTreeSet::new();

        for line in stdout.lines() {
            let package = line.trim();

            if package.is_empty() {
                continue;
            }

            packages.insert(package.to_string());
        }

        Ok(packages)
    }

    pub fn foreign_packages(&self) -> Result<BTreeSet<String>> {
        let output = Command::new("pacman")
            .args(["-Qm"])
            .output()
            .context("could not execute pacman")?;

        if !output.status.success() {
            return Err(anyhow!(
                "pacman -Qm exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let stdout = String::from_utf8(output.stdout).context("pacman produced invalid UTF-8")?;

        let mut packages = BTreeSet::new();

        for line in stdout.lines() {
            let package = line.trim();

            if package.is_empty() {
                continue;
            }

            packages.insert(package.to_string());
        }

        Ok(packages)
    }

    pub fn install(&self, packages: &BTreeSet<String>) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut command = Command::new("pacman");

        command
            .arg("-S")
            .arg("--needed")
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        run_command(command, "pacman -S")
    }

    pub fn remove(&self, packages: &BTreeSet<String>) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        let mut command = Command::new("pacman");

        command
            .arg("-R")
            .args(packages)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        run_command(command, "pacman -R")
    }
}

fn run_command(mut command: Command, description: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("could not execute {description}"))?;

    if !status.success() {
        return Err(anyhow!("{description} failed with {status}"));
    }

    Ok(())
}
