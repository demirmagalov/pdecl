use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;

const AUR_RPC_BASE: &str = "https://aur.archlinux.org/rpc/v5";
const USER_AGENT: &str = "pdecl/0.1.0";

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AurPackage {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub package_base: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    pub num_votes: i64,
    pub popularity: f64,
    pub out_of_date: Option<i64>,
    pub maintainer: Option<String>,
    #[serde(rename = "URLPath")]
    pub url_path: String,

    #[serde(default)]
    pub depends: Vec<String>,

    #[serde(default)]
    pub make_depends: Vec<String>,

    #[serde(default)]
    pub check_depends: Vec<String>,

    #[serde(default)]
    pub opt_depends: Vec<String>,

    #[serde(default)]
    pub conflicts: Vec<String>,

    #[serde(default)]
    pub provides: Vec<String>,

    #[serde(default)]
    pub replaces: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AurResponse {
    version: u32,
    #[serde(rename = "type")]
    response_type: String,
    resultcount: usize,
    results: Vec<AurPackage>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AurClient {
    client: Client,
}

impl AurClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .context("could not create HTTP client")?;

        Ok(Self { client })
    }

    pub fn info(&self, package: &str) -> Result<AurPackage> {
        let response = self
            .client
            .get(format!("{AUR_RPC_BASE}/info"))
            .query(&[("arg[]", package)])
            .send()
            .with_context(|| format!("could not query AUR for {package}"))?
            .error_for_status()
            .with_context(|| format!("AUR returned an HTTP error for {package}"))?;

        let payload: AurResponse = response
            .json()
            .with_context(|| format!("could not parse AUR response for {package}"))?;

        if payload.version != 5 {
            bail!("unsupported AUR RPC version {}", payload.version);
        }

        if payload.response_type == "error" {
            bail!(
                "AUR error for {}: {}",
                package,
                payload
                    .error
                    .unwrap_or_else(|| "unknown AUR error".to_string())
            );
        }

        if payload.resultcount == 0 || payload.results.is_empty() {
            bail!("AUR package {:?} was not found", package);
        }

        payload
            .results
            .into_iter()
            .find(|result| result.name == package)
            .ok_or_else(|| anyhow!("AUR did not return an exact match for {:?}", package))
    }

    pub fn fetch_snapshot(&self, package: &AurPackage, destination: &Path) -> Result<()> {
        fs::create_dir_all(destination)
            .with_context(|| format!("could not create {}", destination.display()))?;

        let url = format!("https://aur.archlinux.org{}", package.url_path);

        let mut response = self
            .client
            .get(&url)
            .send()
            .with_context(|| format!("could not download AUR snapshot for {}", package.name))?
            .error_for_status()
            .with_context(|| format!("AUR snapshot request failed for {}", package.name))?;

        let mut compressed = Vec::new();
        response
            .read_to_end(&mut compressed)
            .with_context(|| format!("could not read AUR snapshot for {}", package.name))?;

        let decoder = GzDecoder::new(compressed.as_slice());
        let mut archive = tar::Archive::new(decoder);

        archive
            .unpack(destination)
            .with_context(|| format!("could not extract AUR snapshot for {}", package.name))?;

        Ok(())
    }

    pub fn fetch_to_store(&self, package: &AurPackage, store: &Path) -> Result<PathBuf> {
        let package_dir = store.join(&package.name);
        let source_dir = package_dir.join("source");

        if source_dir.exists() {
            fs::remove_dir_all(&source_dir).with_context(|| {
                format!("could not remove old source at {}", source_dir.display())
            })?;
        }

        self.fetch_snapshot(package, &source_dir)?;

        Ok(source_dir)
    }
}
