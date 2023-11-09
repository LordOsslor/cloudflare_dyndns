use crate::records::Record;
use clap::Parser;
use core::panic;
use futures::future::join_all;
use simple_logger::SimpleLogger;
use std::{path::PathBuf, sync::Arc};

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
        match cli.config.to_str() {
            Some(v) => v,
            None => "(Non utf-8 string)",
        }
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
        total_search_fields += zone.search.len()
    }

    log::info!(
        "Found configurations for {} zones with {} total search fields",
        &conf.zones.len(),
        total_search_fields
    );

    log::info!("Getting ip addresses");
    let addr = match api::get_ip_addresses(conf.ipv4_service, conf.ipv6_service).await {
        Ok(v) => v,
        Err(e) => panic!("Could not get ip addresses: {}", e),
    };

    log::info!("Got {}", api::address_tuple_to_string(addr));

        log::info!("(\"{id}\"): Listing records");
        let response_list = match api::list_records(&zone).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("Could not list records for zone \"{}\": {}", id, e);
                continue;
            }
        };

        log::info!("(\"{id}\"): Received {} responses", response_list.len());
        log::debug!("(\"{id}\"): Responses: {:?}", response_list);

        let zone_arc = Arc::new(zone);

        let mut futures = Vec::with_capacity(response_list.len());

        log::info!("(\"{id}\"): Patching records");
        for record in response_list {
            let record_arc: Arc<(dyn Record + Send + Sync)> = Arc::new(record);
            let zone_arc_2 = zone_arc.clone();

            futures.push(tokio::spawn(async move {
                let record_name = record_arc.get_name();
                let id = &zone_arc_2.identifier;
                match api::patch_ip_record_address(zone_arc_2.clone(), record_arc.clone(), addr)
                    .await
                {
                    Ok(response) => match response.success {
                        true => {
                            log::info!("(\"{id}\"): ({record_name}): Successfully patched record")
                        }
                        false => log::error!(
                            "(\"{id}\"): ({record_name}): Patch unsuccessful: {:#?}",
                            response.messages
                        ),
                    },
                    Err(e) => log::error!("(\"{id}\"): ({record_name}): {}", e),
                }
            }));
        }
        join_all(futures).await;
    }
}
