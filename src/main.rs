use std::{fs::File, io::Read, sync::Arc};

use crate::records::Record;
use futures::future::join_all;

mod api;
mod config;
mod misc_serialization;
mod records;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[tokio::main]
async fn main() {
    println!("{:?}", built_info::GIT_HEAD_REF);
    println!("{:?}", built_info::GIT_VERSION);
    return;
    let mut config_file = File::open("config.toml").unwrap();
    let mut config_string = String::new();
    config_file.read_to_string(&mut config_string).unwrap();

    let conf: config::Config = toml::from_str(&config_string).unwrap();
    let addr = api::get_ip_addresses(conf.ipv4_service, conf.ipv6_service)
        .await
        .unwrap();
    println!("{:?}", addr);

    for zone in conf.zones {
        let id = zone.identifier.clone();
        println!("Listing records for zone \"{id}\"");
        let response_list = api::list_records(&zone).await.unwrap();

        println!("Received {} responses", response_list.len());

        let zone_arc = Arc::new(zone);

        let mut futures = Vec::with_capacity(response_list.len());

        for record in response_list {
            let record_arc: Arc<(dyn Record + Send + Sync)> = Arc::new(record);
            let zone_arc_2 = zone_arc.clone();

            futures.push(tokio::spawn(async move {
                let record_name = record_arc.get_name();
                match api::patch_ip_record_address(zone_arc_2, record_arc.clone(), addr).await {
                    Ok(r) => match r.success {
                        true => println!("({}): Successfully patched record", record_name),
                        false => println!("({}): Patch unsuccessful", record_name),
                    },
                    Err(r) => println!("({}): Error: {}", record_name, r),
                }
            }));
        }
        join_all(futures).await;
    }
}
