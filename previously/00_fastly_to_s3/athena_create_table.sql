CREATE EXTERNAL TABLE IF NOT EXISTS rubygems.access_logs (
  `timestamp` timestamp,
  `time_elapsed` int,
  `client_ip` string,
  `client_continent` string,
  `client_country` string,
  `client_region` string,
  `client_city` string,
  `client_latitude` string,
  `client_longitude` string,
  `client_timezone` string,
  `client_connection` string,
  `request` string,
  `request_path` string,
  `request_query` string,
  `request_bytes` int,
  `user_agent` string,
  `http2` boolean,
  `tls_version` string,
  `tls_servername` string,
  `tls_cipher` string,
  `response_status` int,
  `response_text` string,
  `response_bytes` int,
  `response_cache` string,
  `cache_state` string,
  `cache_lastuse` float,
  `cache_hits` int,
  `server_region` string,
  `server_datacenter` string 
)
ROW FORMAT SERDE 'org.openx.data.jsonserde.JsonSerDe'
WITH SERDEPROPERTIES (
  'serialization.format' = '1'
) LOCATION 's3://rubygems-logs.rubytogether/'
TBLPROPERTIES ('has_encrypted_data'='false')