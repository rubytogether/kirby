use std::{borrow::Cow, fmt};

use serde::{Deserialize, Deserializer};

use crate::user_agent;

fn default_ip<'a>() -> Cow<'a, str> {
    Cow::from("0.0.0.0")
}

#[derive(Serialize, Debug)]
pub struct ResponseStatus(u16);

impl ResponseStatus {
    pub fn is_success(&self) -> bool {
        self.0 == 200 || self.0 == 304
    }
}

impl<'de> Deserialize<'de> for ResponseStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ResponseStatus;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("integer or string")
            }

            fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                val.try_into().map(ResponseStatus).map_err(E::custom)
            }

            fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match val.parse::<u64>() {
                    Ok(val) => self.visit_u64(val),
                    Err(err) => Err(E::custom(err)),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Shared<'a> {
    #[serde(borrow)]
    pub timestamp: Cow<'a, str>,
    #[serde(borrow)]
    pub request_path: Cow<'a, str>,
    #[serde(borrow)]
    pub request_query: Cow<'a, str>,
    #[serde(borrow, serialize_with = "user_agent::serialize_user_agent")]
    pub user_agent: Cow<'a, str>,
    #[serde(borrow)]
    pub tls_cipher: Cow<'a, str>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Clickhouse<'a> {
    #[serde(flatten)]
    pub shared: Shared<'a>,

    pub time_elapsed: u32,

    #[serde(borrow)]
    pub client_continent: Cow<'a, str>,
    #[serde(borrow)]
    pub client_country: Cow<'a, str>,
    #[serde(borrow)]
    pub client_region: Cow<'a, str>,
    #[serde(borrow)]
    pub client_city: Cow<'a, str>,
    #[serde(borrow)]
    pub client_latitude: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub client_longitude: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub client_timezone: Cow<'a, str>,
    #[serde(borrow)]
    pub client_connection: Cow<'a, str>,

    #[serde(borrow)]
    pub request: Cow<'a, str>,
    #[serde(borrow)]
    pub request_host: Cow<'a, str>,

    pub request_bytes: u32,

    pub http2: bool,
    pub tls: Option<bool>,
    #[serde(borrow)]
    pub tls_version: Cow<'a, str>,

    pub response_status: ResponseStatus,
    #[serde(borrow)]
    pub response_text: Cow<'a, str>,
    pub response_bytes: u32,
    #[serde(borrow)]
    pub response_cache: Cow<'a, str>,
    #[serde(borrow)]
    pub cache_state: Cow<'a, str>,
    pub cache_lastuse: f32,
    pub cache_hits: u32,
    #[serde(borrow)]
    pub server_region: Cow<'a, str>,
    #[serde(borrow)]
    pub server_datacenter: Cow<'a, str>,

    #[serde(borrow, deserialize_with = "empty_string_is_none", default)]
    pub gem: Option<Cow<'a, str>>,
    #[serde(borrow, deserialize_with = "empty_string_is_none", default)]
    pub version: Option<Cow<'a, str>>,
    #[serde(borrow, deserialize_with = "empty_string_is_none", default)]
    pub platform: Option<Cow<'a, str>>,
}

#[derive(Deserialize, Debug)]
pub struct Request<'a> {
    #[serde(flatten)]
    pub shared: Shared<'a>,

    #[serde(borrow)]
    #[serde(default = "default_ip")]
    pub client_ip: Cow<'a, str>,
}

fn empty_string_is_none<'a, D>(deserializer: D) -> Result<Option<Cow<'a, str>>, D::Error>
where
    D: Deserializer<'a>,
{
    let s: Option<Cow<'a, str>> = Deserialize::deserialize(deserializer)?;
    if s.as_ref().is_none_or(|s| s.is_empty()) {
        Ok(None)
    } else {
        Ok(s)
    }
}
