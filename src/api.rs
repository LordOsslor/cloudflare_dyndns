use std::{
    any::type_name,
    collections::HashMap,
    error::Error,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    config::{self, Authorization, SearchRule, Zone},
    records::{ListResponse, PatchResponse, Record, RecordResponse, TypeSpecificData},
};
use futures::{future::join_all, join};
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

async fn list_records_for_rule(
    client_arc: Arc<reqwest::Client>,
    records: Arc<Mutex<HashMap<String, RecordResponse>>>,
    i: usize,
    rule: &SearchRule,
    zone: &Zone,
) -> Result<u32, Box<dyn Error + Sync + Send>> {
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

    let mut request = client_arc
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
    if result.result.is_empty() {
        Err(format!("(Rule {i}): No records returned for search rule"))?;
    }

    let mut new_records = 0;
    {
        let mut record_lock = records.try_lock().unwrap();
        for record in result.result {
            if record_lock.insert(record.id.to_string(), record).is_none() {
                new_records += 1;
            }
        }
    }

    Ok(new_records)
}

pub async fn list_records(
    zone: &config::Zone,
    client_arc: Arc<reqwest::Client>,
) -> Result<HashMap<String, RecordResponse>, Box<dyn Error + Sync + Send>> {
    let mut futures = Vec::new();

    let records = Arc::new(Mutex::new(
        HashMap::<String, RecordResponse>::with_capacity(zone.search.len() * 5),
    ));

    for (i, rule) in zone.search.iter().enumerate() {
        futures.push(list_records_for_rule(
            client_arc.clone(),
            records.clone(),
            i,
            rule,
            zone,
        ));
    }

    let results = join_all(futures).await;
    for future_result in results {
        match future_result {
            Ok(l) => log::debug!(
                "(\"{}\"): Got {} new records from record list",
                zone.identifier,
                l
            ),
            Err(e) => log::error!(
                "(\"{}\"): Error while listing records: {}",
                zone.identifier,
                e
            ),
        }
    }

    Ok(Arc::into_inner(records).unwrap().into_inner().unwrap())
}

pub async fn patch_ip_record_address(
    zone: Arc<config::Zone>,
    record: Arc<dyn Record + Send + Sync>,
    client: Arc<reqwest::Client>,
    addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>),
) -> Result<PatchResponse, Box<dyn Error + Send + Sync>> {
    if addresses.0.is_none() && addresses.0.is_none() {
        return Err("No addresses provided".into());
    }

    let addr = match &record.get_type_data() {
        TypeSpecificData::A { .. } => match addresses.0 {
            Some(a) => Ok(a.to_string()),
            None => Err("No ipv4 address found to patch A record"),
        },
        TypeSpecificData::AAAA { .. } => match addresses.1 {
            Some(a) => Ok(a.to_string()),
            None => Err("No ipv6 address found to patch AAAA record"),
        },
        _ => Err("Provided record is not an ip record"),
    }?;

    let record_id = match record.get_id() {
        Some(id) => Ok(&id.0),
        None => Err("Record does not have an id"),
    }?;
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
        )
        .into()),
    }
}
pub fn address_tuple_to_string(addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>)) -> String {
    match addresses {
        (None, None) => "no addresses".to_owned(),
        (None, Some(v6)) => format!("{} (IPv6)", v6),
        (Some(v4), None) => format!("{} (IPv4)", v4),
        (Some(v4), Some(v6)) => {
            format!("both {} (IPv4) and {} (IPv6)", v4, v6)
        }
    }
}

async fn get_ip_address<T: FromStr>(
    url_opt: Option<String>,
    client: Arc<reqwest::Client>,
) -> Result<Option<T>, Box<dyn Error + Sync + Send>>
where
    <T as FromStr>::Err: Error + Sync + Send,
    <T as FromStr>::Err: 'static,
{
    if let Some(url) = url_opt {
        let ip_version = match type_name::<T>() {
            "core::net::ip_addr::Ipv4Addr" => "IPv4",
            "core::net::ip_addr::Ipv6Addr" => "IPv6",
            t => t,
        };
        log::info!("Getting {ip_version} address");
        let r = client.get(url).send().await.or_else(|e| {
            log::error!("Error while sending get request for {ip_version}: {e}");
            Err(e)
        })?;
        match r.status() {
            StatusCode::OK => Ok(Some({
                let txt = r.text().await.or_else(|e| {
                    log::error!("Error while reading response for {ip_version}: {e}");
                    Err(e)
                })?;
                txt.parse::<T>().or_else(|e| {
                    log::error!("Error while parsing response for {ip_version}: {e}");
                    Err(e)
                })?
            })),
            code => {
                log::error!(
                    "Received status code {} while getting {} address",
                    code,
                    ip_version
                );
                Err("Bad response code".into())
            }
        }
    } else {
        Ok(None)
    }
}

pub async fn get_ip_addresses(
    ipv4_service_url: Option<String>,
    ipv6_service_url: Option<String>,
    client: Arc<reqwest::Client>,
) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>), Box<dyn Error>> {
    let r = join!(
        get_ip_address::<Ipv4Addr>(ipv4_service_url, client.clone()),
        get_ip_address::<Ipv6Addr>(ipv6_service_url, client)
    );

    match r {
        (Err(_), Err(_)) => {
            log::error!("Both IPv4 and IPv6 addresses errored");
            Err("No addresses returned".into())
        }
        (r4, r6) => Ok((r4.unwrap_or(None), r6.unwrap_or(None))),
    }
}

pub async fn patch_zone(
    zone: Zone,
    client_arc: Arc<reqwest::Client>,
    addresses: (Option<Ipv4Addr>, Option<Ipv6Addr>),
) -> Result<u16, Box<dyn Error>> {
    let id = zone.identifier.clone();

    log::info!("(\"{id}\"): Listing records");
    let mut response_map = match list_records(&zone, client_arc.clone()).await {
        Ok(v) => v,
        Err(e) => Err(format!("Could not list records for zone \"{}\": {}", id, e))?,
    };

    log::info!("(\"{id}\"): Received {} records", response_map.len());
    log::debug!("(\"{id}\"): Responses: {:?}", response_map);

    let zone_arc = Arc::new(zone);

    let mut futures = Vec::with_capacity(response_map.len());

    log::info!("(\"{id}\"): Patching records");
    for (_record_id, record) in response_map.drain() {
        if match &record.type_data {
            TypeSpecificData::A { content, .. } => match addresses.0 {
                Some(v) => *content == v.to_string(),
                None => {
                    log::warn!("(\"{id}\"): ({}): Cannot update record as no IPv4 address is provided, skipping",record.name);
                    continue;
                }
            },
            TypeSpecificData::AAAA { content, .. } => match addresses.1 {
                Some(v) => *content == v.to_string(),
                None => {
                    log::warn!("(\"{id}\"): ({}): Cannot update record as no IPv6 address is provided, skipping",record.name);
                    continue;
                }
            },
            _ => {
                log::warn!(
                    "(\"{id}\"): ({}): Record is not an IP record, skipping",
                    record.name
                );
                continue;
            }
        } {
            log::warn!(
                "(\"{id}\"): ({}): Content has not changed, skipping",
                record.name
            );
            continue;
        }

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
                Ok(response) => {
                    if response.success {
                        log::info!("(\"{id}\"): ({record_name}): Successfully patched record");
                    } else {
                        log::error!(
                            "(\"{id}\"): ({record_name}): Patch unsuccessful: {:#?}",
                            response.messages
                        );
                    };
                    response.success
                }
                Err(e) => {
                    log::error!("(\"{id}\"): ({record_name}): {}", e);
                    false
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
