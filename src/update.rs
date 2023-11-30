use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env::current_exe;
use std::error::Error;
#[cfg(target_family = "unix")]
use std::os::unix::prelude::PermissionsExt;
use std::process::Command;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::built_info;

const REPO_OWNER: &str = "LordOsslor";

const GITHUB_LATEST_RELEASE_URL: &str = const_format::formatcp!(
    "https://api.github.com/repos/{REPO_OWNER}/{}/releases/latest",
    built_info::PKG_NAME
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
    fn tag_matches(&self, tag: &str) -> bool {
        tag.contains(&self.tag_name)
    }

    fn get_matching_asset(&self) -> Option<&Asset> {
        for asset in &self.assets {
            let asset_feature_and_target = asset
                .name
                .trim_end_matches(".exe")
                .trim_start_matches("dyndns-update");

            if asset_feature_and_target == built_info::TARGET {
                return Some(asset);
            }
        }
        None
    }
}

async fn get_latest_release(client: &Client) -> Result<Release, reqwest::Error> {
    log::debug!("github latest release url: {}", GITHUB_LATEST_RELEASE_URL);
    client
        .get(GITHUB_LATEST_RELEASE_URL)
        .header("User-Agent", "request")
        .send()
        .await?
        .error_for_status()?
        .json::<Release>()
        .await
}

pub async fn try_update() {
    let version = match built_info::GIT_VERSION {
        Some(version) => version,
        None => {
            log::warn!("Could not get git version from current build metadata");
            return;
        }
    };

    let client = Client::new();

    log::info!("Getting latest release from github");
    let release = match get_latest_release(&client).await {
        Ok(release) => {
            log::info!("Got release {}", release.name);
            log::debug!("Full parsed release data: {:?}", release);
            release
        }
        Err(e) => {
            log::error!(
                "Encountered an error while getting latest release from github: {}",
                e
            );
            return;
        }
    };
    if release.tag_matches(version) {
        log::info!(
            "Already at latest release tag: {} ~= {}",
            release.tag_name,
            version
        );
        return;
    }

    log::info!(
        "Release tag and current tag do not match: {} ~/= {}",
        release.tag_name,
        version
    );
    log::info!("Getting matching executable asset from release");

    let asset = match release.get_matching_asset() {
        Some(asset) => {
            log::info!("Found asset {}", asset.name);
            log::debug!("Full parsed asset data: {:?}", asset);
            asset
        }
        None => {
            log::warn!("Could not find a matching executable asset from latest release");
            return;
        }
    };

    log::info!("Getting current executable path");
    let exe_path = match current_exe() {
        Ok(path) => {
            log::info!(
                "Got executable path {}",
                path.to_str().unwrap_or("COULD NOT DISPLAY PATH")
            );
            path
        }
        Err(e) => {
            log::error!("Error while getting current executable path: {}", e);
            return;
        }
    };

    #[cfg(target_family = "unix")]
    {
        log::info!("(Unix only): Removing current executable");
        match tokio::fs::remove_file(&exe_path).await {
            Ok(_) => log::info!("Successfully removed executable"),
            Err(e) => {
                log::error!("Error while removing current executable: {}", e);
                return;
            }
        };
    }
    #[cfg(target_family = "windows")]
    {
        log::info!("(Windows only): Renaming current executable");

        match tokio::fs::rename(&exe_path, &exe_path.with_extension("old")).await {
            Ok(_) => log::info!("Successfully renamed executable"),
            Err(e) => {
                log::error!("Error while renaming current executable: {}", e);
                return;
            }
        };
    }

    {
        log::info!("Creating new file in place of the old executable");
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .open(&exe_path)
            .await
        {
            Ok(file) => {
                log::info!("Successfully created file");
                file
            }
            Err(e) => {
                log::error!(
                    "Error while creating file in place of the old executable: {}",
                    e
                );
                return;
            }
        };

        #[cfg(target_family = "unix")]
        {
            log::info!("(Unix only): Setting unix file permissions");
            match file.set_permissions(PermissionsExt::from_mode(0o744)).await {
                Ok(_) => log::info!("Successfully set file permissions"),
                Err(e) => {
                    log::error!(
                        "Error while setting unix file permissions for new executable: {}",
                        e
                    );
                    return;
                }
            };
        }

        log::info!(
            "Downloading executable into new file from {}",
            asset.browser_download_url
        );

        match asset.download(&mut file, &client).await {
            Ok(_) => log::info!("Successfully downloaded executable"),
            Err(e) => {
                log::error!("Error while downloading new executable: {}", e);
                return;
            }
        };
    }

    log::info!("Spawning new process from newly downloaded executable");
    match Command::new(exe_path)
        .args(std::env::args().skip(1))
        .arg("--just-updated")
        .envs(std::env::vars())
        .spawn()
    {
        Ok(child) => {
            log::info!("Successfully spawned new process: {}", child.id())
        }
        Err(e) => {
            log::error!(
                "Error while spawning new process from newly downloaded executable: {}",
                e
            );
            return;
        }
    };

    log::warn!("Stopping current process to finish update!");
    std::process::exit(0);
}
