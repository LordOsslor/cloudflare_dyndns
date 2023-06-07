use crate::misc_config::*;
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value;

pub mod records {
    use std::fmt::Debug;
    // use std::collections::HashMap;

    use crate::misc_config::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Meta {
        pub auto_added: Option<bool>,
        pub source: Option<String>,
    }

    pub trait IPRecord: Debug {
        fn get_id(&self) -> &MaxLenString<32>;
        fn get_zone_id(&self) -> &Option<MaxLenString<32>>;
    }

    #[derive(Deserialize, Debug)]
    pub struct ARecord {
        pub content: MaxLenString<255>,
        pub name: MaxLenString<255>,
        pub proxied: Option<bool>,
        pub r#type: String,
        pub comment: Option<String>,
        pub created_on: String,
        pub id: MaxLenString<32>,
        pub locked: bool,
        pub meta: Option<Meta>,
        pub modified_on: String,
        pub proxiable: bool,
        pub tags: Option<Vec<String>>,
        pub ttl: Option<TTLU32>,
        pub zone_id: Option<MaxLenString<32>>,
        pub zone_name: String,
    }
    impl IPRecord for ARecord {
        fn get_id(&self) -> &MaxLenString<32> {
            &self.id
        }
        fn get_zone_id(&self) -> &Option<MaxLenString<32>> {
            &self.zone_id
        }
    }
    #[derive(Deserialize, Debug)]
    pub struct AAAARecord {
        pub content: String,
        pub name: MaxLenString<255>,
        pub proxied: Option<bool>,
        pub r#type: String,
        pub comment: Option<String>,
        pub created_on: String,
        pub id: MaxLenString<32>,
        pub locked: bool,
        pub meta: Option<Meta>,
        pub modified_on: String,
        pub proxiable: bool,
        pub tags: Option<Vec<String>>,
        pub ttl: Option<TTLU32>,
        pub zone_id: Option<MaxLenString<32>>,
        pub zone_name: String,
    }
    impl IPRecord for AAAARecord {
        fn get_id(&self) -> &MaxLenString<32> {
            &self.id
        }
        fn get_zone_id(&self) -> &Option<MaxLenString<32>> {
            &self.zone_id
        }
    }

    //      #[derive(Deserialize, Debug)]
    //      struct CAAData{
    //          flags:Option<u8>,
    //          tag:Option<String>,
    //          value:Option<String>
    //
    //      #[derive(Deserialize, Debug)]
    //      struct CAARecord{
    //          content:String,
    //          data:CAAData,
    //          name:MaxLenString<255>,
    //          r#type:String,
    //          comment:Option<String>,
    //          tags:Option<Vec<String>>,
    //          ttl:Option<TTLU32>
    //      }

    //     #[derive(Deserialize, Debug)]
    //     struct CERTData{
    //         algorithm:Option<u8>,
    //         certificate:Option<String>,
    //         key_tag:Option<u16>,
    //         r#type:Option<u16>
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct CERTRecord{
    //         data:CERTData,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct CNAMERecord{
    //         content:String,
    //         name:MaxLenString<255>,
    //         proxied:Option<bool>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }#[derive(Deserialize,Debug)]
    //     struct DNSKEYData{
    //         algorithm:Option<u8>,
    //         flags:Option<u16>,
    //         protocol:Option<u8>,
    //         public_key:Option<String>
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct DNSKEYRecord{
    //         data:DNSKEYData,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct DSData{
    //         algorithm:Option<u8>,
    //         digest:Option<String>,
    //         digest_type:Option<u8>,
    //         key_tag:Option<u16>
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct DSRecord{
    //         data:DSData,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct HTTPSData{
    //         priority:Option<u16>,
    //         target:Option<String>,
    //         value:Option<String>,
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct HTTPSRecord{
    //         data:HTTPSData,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct LOCRecord{
    //         data:HashMap<String,f32>,   //not bothering to implement this shit
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     } #[derive(Deserialize,Debug)]
    //     struct MXRecord{
    //         content:String,
    //         name:MaxLenString<255>,
    //         priority:u16,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }
    //     #[derive(Deserialize,Debug)]

    //     struct NAPTRData{
    //         flags:Option<String>,
    //         order:Option<u16>,
    //         preference:Option<u16>,
    //         regex:Option<String>,
    //         replacement:Option<String>,
    //         service:Option<String>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct NAPTRRecord{
    //         data:NAPTRData,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct NSRecord{
    //         content:String,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct PTRRecord{
    //         content:String,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct SMIMEADATA{
    //         certificate:Option<String>,
    //         matching_type:Option<u8>,
    //         selector:Option<u8>,
    //         usage:Option<u8>
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct SMIMEARecord{
    //         data:String,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    //     struct SRVData{
    //         name:Option<String>, // Hostname
    //         port:Option<u16>,
    //         priority:Option<u16>,
    //         proto:Option<String>,
    //         service:Option<String>,
    //         target:Option<String>, // Hostname
    //         weight:Option<u16>,
    //     }
    //     #[derive(Deserialize,Debug)]
    //     struct SRVRecord{
    //         data:SRVData,
    //         // name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }

    //     #[derive(Deserialize,Debug)]
    // struct SSHFPDATA{
    //     algorithm:Option<u8>,
    //     fingerprint:Option<>
    // }
    //     #[derive(Deserialize,Debug)]
    //     struct SSHFPRecord{
    //         content:String,
    //         name:MaxLenString<255>,
    //         r#type:String,
    //         comment:Option<String>,
    //         tags:Option<Vec<String>>,
    //         ttl:Option<TTLU32>,
    //     }
}

trait Record {
    fn get_id(&self) -> String;
}

#[derive(Debug)]
pub enum RecordTypes {
    A(records::ARecord),
    AAAA(records::AAAARecord),
    Other,
}

impl<'de> Deserialize<'de> for RecordTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let x: Value = Deserialize::deserialize(deserializer)?;
        let o = x.as_object();
        if let Some(o) = o {
            let t = match o.get_key_value("type") {
                Some(t) => t.1,
                None => return Err("No key 'type' in record object").map_err(D::Error::custom),
            };

            let t: RecordTypes = if let Some(s) = t.as_str() {
                match s {
                    "A" => RecordTypes::A(Deserialize::deserialize(x).map_err(D::Error::custom)?),
                    "AAAA" => {
                        RecordTypes::AAAA(Deserialize::deserialize(x).map_err(D::Error::custom)?)
                    }
                    _ => RecordTypes::Other,
                }
            } else {
                return Err("'type' value is not a string").map_err(D::Error::custom);
            };

            return Ok(t);
        } else {
            Err("No record object").map_err(D::Error::custom)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub code: MinMaxValueU16<1000, { u16::MAX }>,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ResultInfo {
    pub count: u16,
    pub page: u16,
    pub per_page: u16,
    pub total_count: u16,
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub result: Vec<RecordTypes>,

    pub errors: Vec<Message>,
    pub messages: Vec<Message>,

    pub success: bool,
    pub result_info: Option<ResultInfo>,
}

#[derive(Deserialize, Debug)]
pub struct PatchResponse {
    pub result: RecordTypes,

    pub errors: Vec<Message>,
    pub messages: Vec<Message>,

    pub success: bool,
}
