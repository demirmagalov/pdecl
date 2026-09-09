mod aur;
mod cli;
mod config;
mod diff;
mod hash;
mod pacman;
mod store;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{AurCommand, Cli, Command};
use config::load_config;
use diff::{PackageDiff, format_diff};
use pacman::Pacman;
use std::collections::BTreeSet;

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

            let foreign = pacman
                .foreign_packages()
                .context("failed to query foreign packages")?;

            let desired = config.package_set();

            let diff = PackageDiff::between(
                &desired,
                |name| {
                    config
                        .package(name)
                        .map(|package| package.source)
                        .unwrap_or(config::PackageSource::Repo)
                },
                &actual,
                &foreign,
            );

            print!("{}", format_diff(&diff));

            Ok(())
        }

        Command::Check { file } => {
            let config = load_config(&file)?;
            let pacman = Pacman::new();

            let actual = pacman
                .explicit_packages()
                .context("failed to query installed packages")?;

            let foreign = pacman
                .foreign_packages()
                .context("failed to query foreign packages")?;

            let desired = config.package_set();

            let diff = PackageDiff::between(
                &desired,
                |name| {
                    config
                        .package(name)
                        .map(|package| package.source)
                        .unwrap_or(config::PackageSource::Repo)
                },
                &actual,
                &foreign,
            );

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

            let aur_packages = config.aur_packages();

            if !aur_packages.is_empty() {
                anyhow::bail!(
                    "{} declares AUR packages, but AUR installation is not \
                     implemented yet. Phase 2 can inspect and fetch AUR \
                     packages; Phase 4 will handle building/installing them.",
                    aur_packages.iter().cloned().collect::<Vec<_>>().join(", ")
                );
            }

            let pacman = Pacman::new();

            let actual = pacman
                .explicit_packages()
                .context("failed to query installed packages")?;

            let foreign = pacman
                .foreign_packages()
                .context("failed to query foreign packages")?;

            let desired = config.package_set();

            let diff = PackageDiff::between(
                &desired,
                |name| {
                    config
                        .package(name)
                        .map(|package| package.source)
                        .unwrap_or(config::PackageSource::Repo)
                },
                &actual,
                &foreign,
            );

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

            let install: BTreeSet<String> = diff
                .install
                .iter()
                .map(|change| change.name.clone())
                .collect();

            let remove: BTreeSet<String> = diff
                .remove
                .iter()
                .map(|change| change.name.clone())
                .collect();

            if !install.is_empty() {
                println!();
                println!(
                    "Installing {} package{}...",
                    install.len(),
                    if install.len() == 1 { "" } else { "s" }
                );

                pacman
                    .install(&install)
                    .context("pacman failed while installing packages")?;
            }

            if !remove.is_empty() {
                println!();
                println!(
                    "Removing {} package{}...",
                    remove.len(),
                    if remove.len() == 1 { "" } else { "s" }
                );

                pacman
                    .remove(&remove)
                    .context("pacman failed while removing packages")?;
            }

            println!();
            println!("System reconciled successfully.");

            Ok(())
        }

        Command::Aur { command } => run_aur(command),
    }
}

fn run_aur(command: AurCommand) -> Result<()> {
    let client = aur::AurClient::new()?;

    match command {
        AurCommand::Info { package } => {
            let info = client.info(&package)?;

            println!("Package:       {}", info.name);
            println!("Version:       {}", info.version);
            println!("Package base:  {}", info.package_base);

            if let Some(maintainer) = info.maintainer {
                println!("Maintainer:    {maintainer}");
            }

            println!("Votes:         {}", info.num_votes);
            println!("Popularity:    {:.2}", info.popularity);

            if let Some(description) = info.description {
                println!("Description:   {description}");
            }

            if !info.depends.is_empty() {
                println!();
                println!("Dependencies:");

                for dependency in info.depends {
                    println!("  {dependency}");
                }
            }

            if !info.make_depends.is_empty() {
                println!();
                println!("Make dependencies:");

                for dependency in info.make_depends {
                    println!("  {dependency}");
                }
            }

            if !info.check_depends.is_empty() {
                println!();
                println!("Check dependencies:");

                for dependency in info.check_depends {
                    println!("  {dependency}");
                }
            }

            Ok(())
        }

        AurCommand::Fetch { package } => {
            let info = client.info(&package)?;
            let source = client.fetch_to_store(&info, &store::ensure_aur_store()?)?;

            println!("Fetched {}.", info.name);
            println!("Source: {}", source.display());

            Ok(())
        }

        AurCommand::Hash { package } => {
            let info = client.info(&package)?;
            let source = client.fetch_to_store(&info, &store::ensure_aur_store()?)?;

            let source_hash = hash::hash_directory(&source)?;

            println!("Package:     {}", info.name);
            println!("Version:     {}", info.version);
            println!("Source:      {}", source.display());
            println!("Source hash: sha256:{source_hash}");

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
