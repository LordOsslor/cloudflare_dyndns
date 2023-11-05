use std::{fs::File, io::Read};

mod api;
mod config;
mod misc_serialization;
mod records;

#[tokio::main]
async fn main() {
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

        for record in &response_list {
            print!("({}): ", record.name);

            match api::patch_ip_record_address(&zone, Box::new(record), addr).await {
                Ok(r) => match r.success {
                    true => println!("Successfully patched record"),
                    false => println!("Patch unsuccessful"),
                },
                Err(r) => println!("Error: {r}"),
            }
        }
    }
}
