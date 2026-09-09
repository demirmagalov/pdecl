use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "pdecl",
    version,
    about = "Declarative package state for Arch Linux"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the difference between the declaration and the system.
    Diff {
        /// Path to the package declaration.
        #[arg(short, long, default_value = "packages.toml")]
        file: PathBuf,
    },

    /// Check whether the system matches the declaration.
    Check {
        /// Path to the package declaration.
        #[arg(short, long, default_value = "packages.toml")]
        file: PathBuf,
    },

    /// Reconcile the system with the declaration.
    Apply {
        /// Path to the package declaration.
        #[arg(short, long, default_value = "packages.toml")]
        file: PathBuf,

        /// Show what would happen without changing the system.
        #[arg(long)]
        dry_run: bool,

        /// Permit an empty package declaration.
        #[arg(long)]
        allow_empty: bool,
    },

    /// Query and retrieve AUR packages.
    Aur {
        #[command(subcommand)]
        command: AurCommand,
    },

    /// Print the pdecl version.
    Version,
}

#[derive(Debug, Subcommand)]
pub enum AurCommand {
    /// Show metadata for an AUR package.
    Info {
        /// AUR package name.
        package: String,
    },

    /// Fetch an AUR package snapshot into the local pdecl store.
    Fetch {
        /// AUR package name.
        package: String,
    },

    /// Fetch an AUR package and print its deterministic source hash.
    Hash {
        /// AUR package name.
        package: String,
    },
}
