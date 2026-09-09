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
        ///
        /// Without this flag, an empty declaration is rejected because
        /// applying it would attempt to remove every explicitly installed
        /// package.
        #[arg(long)]
        allow_empty: bool,
    },

    /// Print the pdecl version.
    Version,
}
