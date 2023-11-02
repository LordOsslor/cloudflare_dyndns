use std::{fs::File, io::Read};

mod api;
mod config;
mod misc_serialization;
mod records;

#[tokio::main]
async fn main() {
    let mut x = File::open("config.toml").unwrap();
    let mut s = String::new();
    x.read_to_string(&mut s).unwrap();

    let conf: config::Config = toml::from_str(&s).unwrap();
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
