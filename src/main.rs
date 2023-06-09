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

    let x = api::list_records(conf.zones.first().expect("msg"))
        .await
        .unwrap();
    println!("{:?}", x);

    for record in &x {
        let r = api::patch_ip_record_address(conf.zones.first().expect(""), Box::new(record), addr)
            .await
            .unwrap();
        println!("{:?}", r);
    }

    let s = serde_json::to_string(&x).unwrap();
    println!("{}", s);
}
