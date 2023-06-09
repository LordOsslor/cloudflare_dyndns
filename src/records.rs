use serde::{Deserialize, Serialize};

use crate::misc_serialization::{MaxLenString, MinMaxValueU16, TTLU32};

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub auto_added: Option<bool>,
    pub source: Option<String>,
}

mod record_data {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CAAData {
        pub flags: Option<u8>,
        pub tag: Option<String>,
        pub value: Option<String>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct CERTData {
        pub algorithm: Option<u8>,
        pub certificate: Option<String>,
        pub key_tag: Option<u16>,
        pub r#type: Option<u16>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct DNSKEYData {
        pub algorithm: Option<u8>,
        pub flags: Option<u16>,
        pub protocol: Option<u16>,
        pub public_key: Option<String>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct DSData {
        pub algorithm: Option<u8>,
        pub digest: Option<String>,
        pub digest_type: Option<u8>,
        pub key_tag: Option<u16>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct HTTPSData {
        pub priority: Option<u16>,
        pub target: Option<String>,
        pub value: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct URIData {
        pub content: Option<String>,
        pub weight: Option<u16>,
    }
}

use record_data::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TypeSpecificData {
    A {
        content: String,
        proxied: Option<bool>,
    },
    AAAA {
        content: String,
        proxied: Option<bool>,
    },
    CAA {
        #[serde(skip_serializing)]
        content: String,
        data: CAAData,
    },
    CERT {
        #[serde(skip_serializing)]
        content: String,
        data: CERTData,
    },
    CNAME {
        content: String,
    },
    DNSKEY {
        #[serde(skip_serializing)]
        content: String,
        data: DNSKEYData,
    },
    DS {
        #[serde(skip_serializing)]
        content: String,
        data: DSData,
    },
    HTTPS {
        #[serde(skip_serializing)]
        content: String,
        data: HTTPSData,
    },
    LOC {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    MX {
        content: String,
        priority: u16,
    },
    NAPTR {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    NS {
        content: String,
    },
    PTR {
        content: String,
    },
    SMIMEA {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    SRV {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    SSHFP {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    SVCB {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    TLSA {
        #[serde(skip_serializing)]
        content: String,
    }, //todo
    TXT {
        content: String,
    },
    URI {
        #[serde(skip_serializing)]
        content: String,
        data: URIData,
    },
}

pub trait Record {
    fn get_type_data(&self) -> &TypeSpecificData;

    fn get_name(&self) -> &MaxLenString<255>;
    fn get_id(&self) -> Option<&MaxLenString<32>>;

    fn get_comment(&self) -> &Option<String>;

    fn get_tags(&self) -> &Option<Vec<String>>;
    fn get_ttl(&self) -> &Option<TTLU32>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordResponse {
    #[serde(flatten)]
    pub type_data: TypeSpecificData,

    pub name: MaxLenString<255>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing)]
    pub created_on: String,
    pub id: MaxLenString<32>,
    #[serde(skip_serializing)]
    pub locked: bool,
    #[serde(skip_serializing)]
    pub meta: Option<Meta>,
    #[serde(skip_serializing)]
    pub modified_on: String,
    #[serde(skip_serializing)]
    pub proxiable: bool,

    pub tags: Option<Vec<String>>,
    pub ttl: Option<TTLU32>,
    pub zone_id: Option<MaxLenString<32>>,
    pub zone_name: String,
}
impl Record for RecordResponse {
    fn get_comment(&self) -> &Option<String> {
        &self.comment
    }
    fn get_id(&self) -> Option<&MaxLenString<32>> {
        Some(&self.id)
    }
    fn get_name(&self) -> &MaxLenString<255> {
        &self.name
    }
    fn get_tags(&self) -> &Option<Vec<String>> {
        &self.tags
    }
    fn get_ttl(&self) -> &Option<TTLU32> {
        &self.ttl
    }
    fn get_type_data(&self) -> &TypeSpecificData {
        &self.type_data
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
    pub result: Vec<RecordResponse>,

    pub errors: Vec<Message>,
    pub messages: Vec<Message>,

    pub success: bool,
    pub result_info: Option<ResultInfo>,
}

#[derive(Deserialize, Debug)]
pub struct PatchResponse {
    pub result: RecordResponse,

    pub errors: Vec<Message>,
    pub messages: Vec<Message>,

    pub success: bool,
}
