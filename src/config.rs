use crate::misc_serialization::*;

use serde::{Deserialize, Serialize};
use serde_with::with_prefix;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StringMatch {
    pub exact: Option<String>,
    pub absent: Option<bool>,
    pub contains: Option<String>,
    pub endswith: Option<String>,
    pub present: Option<bool>,
    pub startswith: Option<String>,
}
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    asc,
    desc,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Match {
    any,
    all,
}
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Order {
    r#type,
    name,
    content,
    ttl,
    proxied,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum RecordType {
    A,
    AAAA,
    CAA,
    CERT,
    CNAME,
    DNSKEY,
    DS,
    HTTPS,
    LOC,
    MX,
    NAPTR,
    NS,
    PTR,
    SMIMEA,
    SRV,
    SSHFP,
    SVCB,
    TLSA,
    TXT,
    URI,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SearchCriteria {
    #[serde(flatten, with = "prefix_comment")]
    pub comment: Option<StringMatch>,
    pub content: Option<String>,
    #[serde(flatten, with = "prefix_direction")]
    pub direction: Option<Direction>,
    #[serde(flatten, with = "prefix_match")]
    pub r#match: Option<Match>,
    pub name: Option<MaxLenString<255>>,
    #[serde(flatten, with = "prefix_order")]
    pub order: Option<Order>,
    pub page: Option<MinMaxValueU16<1, { u16::MAX }>>,
    pub per_page: Option<MinMaxValueU16<5, 50000>>,
    pub proxied: Option<bool>,
    pub search: Option<String>,
    #[serde(flatten, with = "prefix_tag")]
    pub tag: Option<StringMatch>,
    pub tag_match: Option<String>,
    pub r#type: Option<RecordType>,
}

with_prefix!(prefix_comment "comment.");
with_prefix!(prefix_direction "direction.");
with_prefix!(prefix_match "match.");
with_prefix!(prefix_order "order.");
with_prefix!(prefix_tag "tag.");

#[derive(Serialize, Deserialize)]
pub struct Zone {
    pub identifier: MaxLenString<32>,
    pub auth: Authorization,

    pub search: Vec<SearchCriteria>,
}

#[derive(Serialize, Deserialize)]
pub enum ApiKey {
    Email(String),
    Key(String),
    UserServiceKey(String),
}
impl ApiKey {
    pub fn get_auth_header_tuple(&self) -> (String, String) {
        match self {
            ApiKey::Email(token) => ("X-Auth-Email".to_owned(), token.to_string()),
            ApiKey::Key(token) => ("X-Auth-Key".to_owned(), token.to_string()),
            ApiKey::UserServiceKey(token) => {
                ("X-Auth-User-Service-Key".to_owned(), token.to_string())
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
pub enum Authorization {
    BearerAuth(String),
    ApiKey(ApiKey),
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub ipv4_service: Option<String>,
    pub ipv6_service: Option<String>,

    pub zones: Vec<Zone>,
}
