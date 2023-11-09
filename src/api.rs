use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

use crate::{
    config::{self, Authorization},
    records::{ListResponse, PatchResponse, Record, RecordResponse, TypeSpecificData},
};
use reqwest::{Method, RequestBuilder, StatusCode};

fn authenticate_request(req: RequestBuilder, auth: &Authorization) -> RequestBuilder {
    match auth {
        Authorization::BearerAuth(token) => req.bearer_auth(token),
        Authorization::ApiKey(api_key) => {
            let (key, value) = api_key.get_auth_header_tuple();
            req.header(key, value)
        }
    }
}

pub async fn list_records(
    zone: &config::Zone,
) -> Result<Vec<RecordResponse>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut record_vec = Vec::<RecordResponse>::new();
    for rule in &zone.search {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?{}",
            zone.identifier.0,
            serde_url_params::to_string(&rule)?
        );

        let mut request = client
            .request(Method::GET, url)
            .header("Content-Type", "application/json");

        request = authenticate_request(request, &zone.auth);

        let response = request.send().await?;

        let status = response.status();
        let text = response.text().await?;

        let mut result: ListResponse = match status {
            StatusCode::OK => serde_json::from_str(&text)?,
            code => Err(format!(
                "Response for list records request is of code: {}\nText: {}",
                code, text
            ))?,
        };

        record_vec.append(&mut result.result);
    }

    Ok(record_vec)
}

pub async fn patch_ip_record_address(
    zone: Arc<config::Zone>,
    record: Arc<dyn Record + Send + Sync>,
    addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>),
) -> Result<PatchResponse, Box<dyn std::error::Error + Send + Sync>> {
    if addresses.0.is_none() && addresses.0.is_none() {
        Err("No addresses provided")?
    }

    let client = reqwest::Client::new();

    let addr = match &record.get_type_data() {
        TypeSpecificData::A { .. } => match addresses.0 {
            Some(a) => a.to_string(),
            None => Err("No ipv4 address found to patch A record")?,
        },
        TypeSpecificData::AAAA { .. } => match addresses.1 {
            Some(a) => a.to_string(),
            None => Err("No ipv6 address found to patch AAAA record")?,
        },
        _ => Err("Provided record is not an ip record")?,
    };

    let record_id = match record.get_id() {
        Some(id) => &id.0,
        None => Err("Record does not have an id")?,
    };
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone.identifier, record_id
    );
    let mut req = client.patch(url).header("Content-Type", "application/json");

    req = authenticate_request(req, &zone.auth);
    let mut map = HashMap::new();
    map.insert("content", addr);
    req = req.json(&map);

    let response = req.send().await?;

    let status = response.status();
    let text = response.text().await?;

    match status {
        StatusCode::OK => Ok(serde_json::from_str(&text)?),
        code => Err(format!(
            "Error {} while patching record {}: {}",
            code,
            &record.get_name().0,
            text
        ))?,
    }
}
pub fn address_tuple_to_string(addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>)) -> String {
    match addresses {
        (None, None) => "no addresses".to_owned(),
        (None, Some(v6)) => format!("{} (IPv6)", v6.to_string()),
        (Some(v4), None) => format!("{} (IPv4)", v4.to_string()),
        (Some(v4), Some(v6)) => {
            format!(
                "both {} (IPv4) and {} (IPv6)",
                v4.to_string(),
                v6.to_string()
            )
        }
    }
}

pub async fn get_ip_addresses(
    ipv4_service_url: Option<String>,
    ipv6_service_url: Option<String>,
) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), Box<dyn std::error::Error>> {
    if ipv4_service_url.is_none() && ipv6_service_url.is_none() {
        Err("No ip service set")?
    }

    let ipv4_addr = if let Some(ipv4_service_url) = ipv4_service_url {
        let resp4 = reqwest::get(ipv4_service_url).await?;
        match resp4.status() {
            StatusCode::OK => match resp4.text().await?.parse() {
                Ok(v) => Some(v),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };

    let ipv6_addr = if let Some(ipv6_service_url) = ipv6_service_url {
        let resp6 = reqwest::get(ipv6_service_url).await?;
        match resp6.status() {
            StatusCode::OK => match resp6.text().await?.parse() {
                Ok(v) => Some(v),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };

    Ok((ipv4_addr, ipv6_addr))
}
