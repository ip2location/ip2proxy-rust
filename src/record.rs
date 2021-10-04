use serde::Serialize;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Debug, Clone, Serialize)]
// pub struct IP2ProxyRecord {
pub struct Record {
    pub ip: String,
    pub is_proxy: i8,
    pub country_short: String,
    pub country_long: String,
    pub region: String,
    pub city: String,
    pub isp: String,
    pub proxy_type: String,
    pub usage_type: String,
    pub as_name: String,
    pub asn: String,
    pub last_seen: String,
    pub domain: String,
    pub threat: String,
    pub provider: String,

}

impl Record {
    pub fn new_empty() -> Self {
        Self {
            ip: "".to_string(),
            is_proxy: -1,
            country_short: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            country_long: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            region: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            city: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            isp: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            proxy_type: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            usage_type: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            as_name: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            asn: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            last_seen: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            domain: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            threat: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
            provider: "This parameter is unavailable for selected data file. Please upgrade the data file.".to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}