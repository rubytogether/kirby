use std::borrow::Cow;

fn default_ip<'a>() -> Cow<'a, str> {
  Cow::from("0.0.0.0")
}

#[derive(Deserialize, Debug)]
pub struct Request<'a> {
  #[serde(borrow)]
  pub timestamp: Cow<'a, str>,
  // time_elapsed: u8,
  #[serde(borrow)]
  #[serde(default="default_ip")]
  pub client_ip: Cow<'a, str>,
  // client_continent: String,
  // client_country: String,
  // client_region: String,
  // client_city: String,
  // client_latitude: String,
  // client_longitude: String,
  // pub client_timezone: String,
  // client_connection: String,
  // request: String,
  // request_host: String,
  #[serde(borrow)]
  pub request_path: Cow<'a, str>,
  #[serde(borrow)]
  pub request_query: Cow<'a, str>,
  // request_bytes: u16,
  #[serde(borrow)]
  pub user_agent: Cow<'a, str>,
  // pub http2: bool,
  // pub tls: Option<bool>,
  // #[serde(borrow)]
  // pub tls_version: Cow<'a, str>,
  #[serde(borrow)]
  pub tls_cipher: Cow<'a, str>,
  // response_status: String,
  // response_text: String,
  // response_bytes: u16,
  // response_cache: String,
  // cache_state: String,
  // cache_lastuse: f32,
  // cache_hits: u16,
  // #[serde(borrow)]
  // pub server_region: Cow<'a, str>,
  // server_datacenter: String,
}
