use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env::current_exe;
use std::error::Error;
#[cfg(target_family = "unix")]
use std::os::unix::prelude::PermissionsExt;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::built_info::TARGET;

const REPO_OWNER: &str = "LordOsslor";
const REPO_NAME: &str = "dyndns";

const GITHUB_LATEST_RELEASE_URL: &str = const_format::formatcp!(
    "https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest"
);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Asset {
    url: String,
    name: String,
    browser_download_url: String,
}
impl Asset {
    async fn download(&self, file: &mut File, client: &Client) -> Result<usize, Box<dyn Error>> {
        let response = client
            .get(&self.browser_download_url)
            .header("User-Agent", "request")
            .send()
            .await?;

        let mut stream = response.bytes_stream();

        let mut total_size: usize = 0;
        while let Some(chunk) = stream.next().await {
            total_size += file.write(&chunk?).await?;
        }

        Ok(total_size)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Release {
    url: String,
    tag_name: String,
    name: String,
    assets: Vec<Asset>,
}
impl Release {
    fn tag_differs(&self, tag: &str) -> bool {
        !tag.contains(&self.tag_name)
    }

    fn get_matching_asset(&self) -> Option<&Asset> {
        for asset in &self.assets {
            if asset.name.contains(TARGET) {
                return Some(asset);
            }
        }
        None
    }
}

async fn get_latest_release(client: &Client) -> Result<Release, reqwest::Error> {
    client
        .get(GITHUB_LATEST_RELEASE_URL)
        .header("User-Agent", "request")
        .send()
        .await?
        .error_for_status()?
        .json::<Release>()
        .await
}

pub async fn update_if_not_latest_release(tag: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    println!("Checking for new release");

    let client = Client::new();

    let release = get_latest_release(&client).await?;

    println!(
        "Latest release: {}, Installed version: {}, Differs: {}",
        release.tag_name,
        tag,
        release.tag_differs(tag)
    );

    if !release.tag_differs(tag) {
        Err(format!(
            "Already at most recent tag ({} ~= {})",
            release.tag_name, tag
        ))?
    }

    println!("Trying to get binary from latest release");

    let asset = release.get_matching_asset().ok_or_else(|| {
        format!(
            "No matching binary could be found for target {} in release for tag {}",
            TARGET, release.tag_name
        )
    })?;

    let exe_path = current_exe()?;
    #[cfg(target_family = "unix")]
    {
        println!("Removing current binary");
        tokio::fs::remove_file(&exe_path).await?;
    }
    #[cfg(target_family = "windows")]
    {
        println!("Renaming current binary (.old)");
        tokio::fs::rename(&exe_path, &exe_path.with_extension("old")).await?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&exe_path)
        .await?;

    #[cfg(target_family = "unix")]
    {
        println!("Setting unix file permissions");
        file.set_permissions(PermissionsExt::from_mode(0o744))
            .await?;
    }

    println!("Downloading new binary");
    asset.download(&mut file, &client).await?;

    Ok(exe_path)
}
