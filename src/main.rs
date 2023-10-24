use std::{fs::File, io::Read};

use crate::api::patch_all_matching_zone_ip_records;

mod api;
mod config;
mod misc_serialization;
mod records;

#[tokio::main]
async fn main() {
    //Read Config
    let mut x = File::open("config.toml").unwrap();
    let mut s = String::new();
    x.read_to_string(&mut s).unwrap();

    //Get ip address
    let conf: config::Config = toml::from_str(&s).unwrap();
    let addr = api::get_ip_addresses(conf.ipv4_service, conf.ipv6_service)
        .await
        .unwrap();
    println!("{:?}", addr);

    let x = patch_all_matching_zone_ip_records(conf.zones.first().unwrap(), addr)
        .await
        .unwrap();
}
