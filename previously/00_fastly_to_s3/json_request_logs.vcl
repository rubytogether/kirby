call json_generate_reset;
call json_generate_begin_object;

set req.http.value = "timestamp";
call json_generate_string;
set req.http.value = strftime("%25F %25T", time.start);
call json_generate_string;

set req.http.value = "time_elapsed";
call json_generate_string;
set req.http.value = time.elapsed.msec;
call json_generate_number;

set req.http.value = "client_ip";
call json_generate_string;
set req.http.value = req.http.Fastly-Client-IP;
call json_generate_string;

set req.http.value = "client_continent";
call json_generate_string;
set req.http.value = client.geo.continent_code;
call json_generate_string;

set req.http.value = "client_country";
call json_generate_string;
set req.http.value = client.geo.country_name.utf8;
call json_generate_string;

set req.http.value = "client_region";
call json_generate_string;
set req.http.value = client.geo.region;
call json_generate_string;

set req.http.value = "client_city";
call json_generate_string;
set req.http.value = client.geo.city.utf8;
call json_generate_string;

set req.http.value = "client_latitude";
call json_generate_string;
set req.http.value = client.geo.latitude;
call json_generate_string;

set req.http.value = "client_longitude";
call json_generate_string;
set req.http.value = client.geo.longitude;
call json_generate_string;

set req.http.value = "client_timezone";
call json_generate_string;
set req.http.value = client.geo.gmt_offset;
call json_generate_string;

set req.http.value = "client_connection";
call json_generate_string;
set req.http.value = client.geo.conn_speed;
call json_generate_string;

set req.http.value = "request";
call json_generate_string;
set req.http.value = req.request;
call json_generate_string;

set req.http.value = "request_host";
call json_generate_string;
set req.http.value = req.http.host;
call json_generate_string;

set req.http.value = "request_path";
call json_generate_string;
set req.http.value = req.url.path;
call json_generate_string;

set req.http.value = "request_query";
call json_generate_string;
set req.http.value = req.url.qs;
call json_generate_string;

set req.http.value = "request_bytes";
call json_generate_string;
set req.http.value = req.bytes_read;
call json_generate_number;

set req.http.value = "user_agent";
call json_generate_string;
set req.http.value = req.http.User-Agent;
call json_generate_string;

set req.http.value = "http2";
call json_generate_string;
set req.http.value = fastly_info.is_h2;
call json_generate_bool;

set req.http.value = "tls_version";
call json_generate_string;
set req.http.value = tls.client.protocol;
call json_generate_string;

set req.http.value = "tls_cipher";
call json_generate_string;
set req.http.value = tls.client.cipher;
call json_generate_string;

set req.http.value = "response_status";
call json_generate_string;
set req.http.value = resp.status;
call json_generate_string;

set req.http.value = "response_text";
call json_generate_string;
set req.http.value = resp.response;
call json_generate_string;

set req.http.value = "response_bytes";
call json_generate_string;
set req.http.value = resp.bytes_written;
call json_generate_number;

set req.http.value = "response_cache";
call json_generate_string;
set req.http.value = resp.http.X-Cache;
call json_generate_string;

set req.http.value = "cache_state";
call json_generate_string;
set req.http.value = fastly_info.state;
call json_generate_string;

set req.http.value = "cache_lastuse";
call json_generate_string;
set req.http.value = obj.lastuse;
call json_generate_number;

set req.http.value = "cache_hits";
call json_generate_string;
set req.http.value = obj.hits;
call json_generate_number;

set req.http.value = "server_region";
call json_generate_string;
set req.http.value = server.region;
call json_generate_string;

set req.http.value = "server_datacenter";
call json_generate_string;
set req.http.value = server.datacenter;
call json_generate_string;

call json_generate_end_object;
log {"syslog 2pDNderOIzEhIsrKfwWurj s3-json :: "} req.http.json_generate_json;