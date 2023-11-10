use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

use crate::{
    config::{self, Authorization, Zone},
    records::{ListResponse, PatchResponse, Record, RecordResponse, TypeSpecificData},
};
use futures::future::join_all;
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
    client_arc: Arc<reqwest::Client>,
) -> Result<Vec<RecordResponse>, Box<dyn std::error::Error>> {
    let mut futures = Vec::new();

    for (i, rule) in zone.search.iter().enumerate() {
        let client_arc_2 = client_arc.clone();
        futures.push(async move {
            let url_params = match serde_url_params::to_string(&rule) {
                Ok(v) => v,
                Err(e) => Err(format!(
                    "(Rule {i}): Search rule could not be serialized into url parameters: {e}",
                ))?,
            };

            let url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records?{}",
                zone.identifier.0, url_params
            );

            let mut request = client_arc_2
                .clone()
                .request(Method::GET, url)
                .header("Content-Type", "application/json");

            request = authenticate_request(request, &zone.auth);

            let response = request.send().await?;

            let status = response.status();
            let text = response.text().await?;

            let result: ListResponse = match status {
                StatusCode::OK => serde_json::from_str(&text)?,
                code => Err(format!(
                    "(Rule {i}): Response for list records request is of code: {}; Text: {}",
                    code, text
                ))?,
            };
            if result.result.len() == 0 {
                Err(format!("(Rule {i}): No records returned for search rule"))?
            }
            Ok::<ListResponse, Box<dyn std::error::Error>>(result)
        });
    }

    let mut record_vec = Vec::<RecordResponse>::new();
    let results = join_all(futures).await;
    for r in results {
        match r {
            Ok(mut r) => record_vec.append(&mut r.result),
            Err(e) => log::error!(
                "(\"{}\"): Error while listing records: {}",
                zone.identifier,
                e
            ),
        }
    }

    Ok(record_vec)
}

pub async fn patch_ip_record_address(
    zone: Arc<config::Zone>,
    record: Arc<dyn Record + Send + Sync>,
    client: Arc<reqwest::Client>,
    addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>),
) -> Result<PatchResponse, Box<dyn std::error::Error + Send + Sync>> {
    if addresses.0.is_none() && addresses.0.is_none() {
        Err("No addresses provided")?
    }

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

fn ip_type_and_content_match(
    type_data: &TypeSpecificData,
    addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>),
) -> Result<(bool, String), &str> {
    match type_data {
        TypeSpecificData::A { content, .. } => match addresses.0 {
            Some(a) => Ok((*content == a.to_string(), a.to_string())),
            None => Err("No ipv4 address found to patch A record")?,
        },
        TypeSpecificData::AAAA { content, .. } => match addresses.1 {
            Some(a) => Ok((*content == a.to_string(), a.to_string())),
            None => Err("No ipv6 address found to patch AAAA record")?,
        },
        _ => Err("Provided record is not an ip record"),
    }
}

pub async fn patch_zone(
    zone: Zone,
    client_arc: Arc<reqwest::Client>,
    addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>),
) -> Result<u16, Box<dyn std::error::Error>> {
    let id = zone.identifier.clone();

    log::info!("(\"{id}\"): Listing records");
    let response_list = match list_records(&zone, client_arc.clone()).await {
        Ok(v) => v,
        Err(e) => Err(format!("Could not list records for zone \"{}\": {}", id, e))?,
    };

    log::info!("(\"{id}\"): Received {} responses", response_list.len());
    log::debug!("(\"{id}\"): Responses: {:?}", response_list);

    let zone_arc = Arc::new(zone);

    let mut futures = Vec::with_capacity(response_list.len());

    log::info!("(\"{id}\"): Patching records");
    for record in response_list {
        match ip_type_and_content_match(&record.type_data, addresses)? {
            (true, content) => {
                log::warn!(
                    "(\"{id}\"): ({}): Content has not changed, skipping: {}",
                    record.name,
                    content
                );
                continue;
            }
            (false, _) => log::debug!("(\"{id}\"): ({}): Content has changed", record.name),
        };

        let record_arc: Arc<(dyn Record + Send + Sync)> = Arc::new(record);
        let client_arc_2 = client_arc.clone();
        let zone_arc_2 = zone_arc.clone();

        futures.push(tokio::spawn(async move {
            let id = &zone_arc_2.identifier;
            let record_name = record_arc.get_name();

            match patch_ip_record_address(
                zone_arc_2.clone(),
                record_arc.clone(),
                client_arc_2,
                addresses,
            )
            .await
            {
                Ok(response) => match response.success {
                    true => {
                        log::info!("(\"{id}\"): ({record_name}): Successfully patched record");
                        return true;
                    }
                    false => {
                        log::error!(
                            "(\"{id}\"): ({record_name}): Patch unsuccessful: {:#?}",
                            response.messages
                        );
                        return false;
                    }
                },
                Err(e) => {
                    log::error!("(\"{id}\"): ({record_name}): {}", e);
                    return false;
                }
            }
        }));
    }
    let mut success_count = 0;
    for r in join_all(futures).await {
        if r? {
            success_count += 1;
        }
    }

    Ok(success_count)
}
