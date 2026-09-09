mod cli;
mod config;
mod diff;
mod pacman;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Command};
use config::load_config;
use diff::{PackageDiff, format_diff};
use pacman::Pacman;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Version => {
            println!("pdecl {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        Command::Diff { file } => {
            let config = load_config(&file)?;
            let pacman = Pacman::new();

            let actual = pacman
                .explicit_packages()
                .context("failed to query installed packages")?;

            let desired = config.package_set();
            let diff = PackageDiff::between(&desired, &actual);

            print!("{}", format_diff(&diff));

            Ok(())
        }

        Command::Check { file } => {
            let config = load_config(&file)?;
            let pacman = Pacman::new();

            let actual = pacman
                .explicit_packages()
                .context("failed to query installed packages")?;

            let desired = config.package_set();
            let diff = PackageDiff::between(&desired, &actual);

            if diff.is_empty() {
                println!("System matches {}.", file.display());
                return Ok(());
            }

            println!("System does not match {}.", file.display());
            println!();
            print!("{}", format_diff(&diff));

            std::process::exit(1);
        }

        Command::Apply {
            file,
            dry_run,
            allow_empty,
        } => {
            let config = load_config(&file)?;

            if config.packages.is_empty() && !allow_empty {
                anyhow::bail!(
                    "{} declares zero packages. Refusing to remove every \
                     explicitly installed package. If this is intentional, \
                     use --allow-empty.",
                    file.display()
                );
            }

            let pacman = Pacman::new();

            let actual = pacman
                .explicit_packages()
                .context("failed to query installed packages")?;

            let desired = config.package_set();
            let diff = PackageDiff::between(&desired, &actual);

            if diff.is_empty() {
                println!("System already matches {}.", file.display());
                return Ok(());
            }

            print!("{}", format_diff(&diff));

            if dry_run {
                println!("Dry run: no changes made.");
                return Ok(());
            }

            println!();

            if !confirm()? {
                println!("Aborted.");
                return Ok(());
            }

            if !diff.install.is_empty() {
                println!();
                println!(
                    "Installing {} package{}...",
                    diff.install.len(),
                    if diff.install.len() == 1 {
                        ""
                    } else {
                        "s"
                    }
                );

                pacman
                    .install(&diff.install)
                    .context("pacman failed while installing packages")?;
            }

            if !diff.remove.is_empty() {
                println!();
                println!(
                    "Removing {} package{}...",
                    diff.remove.len(),
                    if diff.remove.len() == 1 {
                        ""
                    } else {
                        "s"
                    }
                );

                pacman
                    .remove(&diff.remove)
                    .context("pacman failed while removing packages")?;
            }

            println!();
            println!("System reconciled successfully.");

            Ok(())
        }
    }
}

fn confirm() -> Result<bool> {
    use std::io::{self, Write};

    print!("Apply these changes? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
