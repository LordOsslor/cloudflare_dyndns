#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use clap::Parser;
use core::panic;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
mod api;
mod config;
mod misc_serialization;
mod records;
#[cfg(feature = "update")]
mod update;
#[cfg(feature = "update")]
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short,long,default_value=clap::builder::OsStr::from("config.toml"))]
    config: PathBuf,
    #[cfg(feature = "update")]
    #[arg(short, long)]
    update: bool,
    #[cfg(feature = "update")]
    #[arg(long)]
    just_updated: bool,
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .expect("Logger should be initializable in main function");

    log::debug!("Parsing CLI args");
    let cli = CliArgs::parse();
    log::debug!("CLI Args: {:?}", cli);

    #[cfg(feature = "update")]
    {
        log::debug!("Update feature enabled");
        if cli.update {
            log::info!("Update cli flag set. Trying to update:");
            update::try_update().await;
        }
    }

    log::info!(
        "Opening config file at {}",
        cli.config.to_str().unwrap_or("(Non utf-8 string)")
    );
    let mut config_file = match File::open(cli.config).await {
        Ok(file) => file,
        Err(e) => panic!("Could not open config file {}", e),
    };
    let mut config_string = String::new();
    match config_file.read_to_string(&mut config_string).await {
        Ok(_) => (),
        Err(e) => panic!("Could not read config file: {}", e),
    };

    let conf: config::Config = match toml::from_str(&config_string) {
        Ok(v) => v,
        Err(e) => panic!("Could not parse config file: {}", e),
    };

    let mut total_search_fields = 0;
    for zone in &conf.zones {
        total_search_fields += zone.search.len();
    }

    log::info!(
        "Found configurations for {} zones with {} total search rules",
        &conf.zones.len(),
        total_search_fields
    );

    let client = Arc::new(reqwest::Client::new());
    log::info!("Getting ip addresses");
    let addr =
        match api::get_ip_addresses(conf.ipv4_service, conf.ipv6_service, client.clone()).await {
            Ok(v) => v,
            Err(e) => panic!("Could not get ip addresses: {}", e),
        };

    log::info!("Got {}", api::address_tuple_to_string(addr));

    for zone in conf.zones {
        let id = &zone.identifier.clone();
        match api::patch_zone(zone, client.clone(), addr).await {
            Ok(i) => log::info!("(\"{id}\"): Patched {i} records"),
            Err(e) => log::error!("\"{id}\": Fatal error while patching records: {e}"),
        };
    }
}
