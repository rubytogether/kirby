## Request Line

field|type|description
---|---|---
timestamp|DateTime|Timestamp of the event
request_path|String|Path of the request
request_query|String|Query string of the request
user_agent|UserAgent|User agent of the client
tls_cipher|String|Cipher used for the connection
time_elapsed|u32|Time elapsed for the request
client_continent|String|Continent of the client
client_country|String|Country of the client
client_region|String|Region of the client
client_city|String|City of the client
client_latitude|nullable String|Latitude of the client
client_longitude|nullable String|Longitude of the client
client_timezone|String|Timezone of the client
client_connection|String|Connection type of the client
request|String|Request method
request_host|String|Host of the request
request_bytes|u32|Bytes sent in the request
http2|bool|HTTP/2 support
tls|nullable bool|whether the connection was encrypted
tls_version|String|TLS version used
response_status|u32|HTTP status code
response_text|String|HTTP status text
response_bytes|u32|Bytes sent in the response
response_cache|low cardinality String|Cache status of the response
cache_state|low cardinality String|Cache state of the response
cache_lastuse|f32|Time since the cache was last used
cache_hits|u32|Number of cache hits
server_region|low cardinality String|Region of the server
server_datacenter|low cardinality String|Datacenter of the server
gem|String|Name of the downloaded gem
version|String|Version of the downloaded gem
platform|Platform|Platform of the downloaded gem

## UserAgent

null values will be omitted

field|type|description
---|---|---
agent_name|nullable String|Name of the user agent
agent_version|nullable String|Version of the user agent
bundler|nullable String|Bundler version
rubygems|nullable String|Rubygems version
ruby|nullable String|Ruby version
platform|nullable Platform|Platform
command|nullable String|Bundler command run
options|nullable String|Options passed to the command
jruby|nullable String|JRuby version
truffleruby|nullable String|TruffleRuby version
ci|nullable String|CI system(s), comma separated
gemstash|nullable String|Gemstash version

## Platform

field|type|description
---|---|---
cpu|low cardinality nullable String|CPU architecture
os|low cardinality nullable String|Operating system
os_version|nullable String|Operating system version