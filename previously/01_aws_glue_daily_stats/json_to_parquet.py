import sys
from awsglue.transforms import *
from awsglue.utils import getResolvedOptions
from pyspark.context import SparkContext
from awsglue.context import GlueContext
from awsglue.job import Job

import re
bundler_matcher = r"\Abundler\/(?P<bundler>[0-9a-z.\-]+) rubygems\/(?P<rubygems>[0-9a-z.\-]+) ruby\/(?P<ruby>[0-9a-z.\-]+) \((?P<platform>.*)\) command\/(?P<bundler_command>(.*?))( jruby\/(?P<jruby>[0-9a-z.\-]+))?( truffleruby\/(?P<truffleruby>[0-9a-z.\-]+))?( options\/(?P<bundler_options>\S.*)?)?( ci\/(?P<ci>\S.*)?)? (?P<bundler_command_uid>.*)\Z"
rubygems_matcher = r"\A(Ruby, )?RubyGems\/(?P<rubygems>[0-9a-z.\-]+) (?P<platform>.*) Ruby\/(?P<ruby>[0-9a-z.\-]+) \((?P<ruby_build>.*?)\)( jruby|truffleruby)?( Gemstash\/(?P<gemstash>[0-9a-z.\-]+))?\Z"

def parse_user_agent(rec):
  bundler = re.match(bundler_matcher, rec["user_agent"])
  if bundler:
    fields = ["bundler", "rubygems", "ruby", "platform", "bundler_command", "jruby", "bundler_options", "ci", "bundler_command_uid"]
    for name in fields:
      if bundler.group(name):
        rec[name] = bundler.group(name)
  rubygems = re.match(rubygems_matcher, rec["user_agent"])
  if rubygems:
    fields = ["rubygems", "platform", "ruby", "ruby_build", "gemstash"]
    for name in fields:
      if rubygems.group(name):
        rec[name] = rubygems.group(name)
  return rec

def break_out_date(rec):
  t = rec["timestamp"]
  rec["year"] = t[0:4]
  rec["month"] = t[5:7]
  rec["day"] = t[8:10]
  return rec

def map_function(rec):
  return break_out_date(parse_user_agent(rec))

import random
def filter_function(rec):
  return random.randint(1, 1000) == 1

## @params: [JOB_NAME]
args = getResolvedOptions(sys.argv, ['JOB_NAME'])

sc = SparkContext()
glueContext = GlueContext(sc)
spark = glueContext.spark_session
job = Job(glueContext)
job.init(args['JOB_NAME'], args)
## @type: DataSource
## @args: [database = "rubygems", table_name = "fastly_json", transformation_ctx = "datasource0"]
## @return: datasource0
## @inputs: []
datasource0 = glueContext.create_dynamic_frame.from_catalog(database = "rubygems", table_name = "fastly_json", transformation_ctx = "datasource0")
## @type: ApplyMapping
## @args: [mapping = [("timestamp", "string", "timestamp", "string"), ("time_elapsed", "int", "time_elapsed", "int"), ("client_ip", "string", "client_ip", "string"), ("client_continent", "string", "client_continent", "string"), ("client_country", "string", "client_country", "string"), ("client_region", "string", "client_region", "string"), ("client_city", "string", "client_city", "string"), ("client_latitude", "string", "client_latitude", "string"), ("client_longitude", "string", "client_longitude", "string"), ("client_timezone", "string", "client_timezone", "string"), ("client_connection", "string", "client_connection", "string"), ("request", "string", "request", "string"), ("request_host", "string", "request_host", "string"), ("request_path", "string", "request_path", "string"), ("request_query", "string", "request_query", "string"), ("request_bytes", "int", "request_bytes", "int"), ("user_agent", "string", "user_agent", "string"), ("http2", "boolean", "http2", "boolean"), ("tls", "boolean", "tls", "boolean"), ("tls_version", "string", "tls_version", "string"), ("tls_cipher", "string", "tls_cipher", "string"), ("response_status", "string", "response_status", "string"), ("response_text", "string", "response_text", "string"), ("response_bytes", "int", "response_bytes", "int"), ("response_cache", "string", "response_cache", "string"), ("cache_state", "string", "cache_state", "string"), ("cache_lastuse", "double", "cache_lastuse", "double"), ("cache_hits", "int", "cache_hits", "int"), ("server_region", "string", "server_region", "string"), ("server_datacenter", "string", "server_datacenter", "string")], transformation_ctx = "applymapping1"]
## @return: applymapping1
## @inputs: [frame = datasource0]
applymapping1 = ApplyMapping.apply(frame = datasource0, mappings = [("timestamp", "string", "timestamp", "string"), ("time_elapsed", "int", "time_elapsed", "int"), ("client_ip", "string", "client_ip", "string"), ("client_continent", "string", "client_continent", "string"), ("client_country", "string", "client_country", "string"), ("client_region", "string", "client_region", "string"), ("client_city", "string", "client_city", "string"), ("client_latitude", "string", "client_latitude", "string"), ("client_longitude", "string", "client_longitude", "string"), ("client_timezone", "string", "client_timezone", "string"), ("client_connection", "string", "client_connection", "string"), ("request", "string", "request", "string"), ("request_host", "string", "request_host", "string"), ("request_path", "string", "request_path", "string"), ("request_query", "string", "request_query", "string"), ("request_bytes", "int", "request_bytes", "int"), ("user_agent", "string", "user_agent", "string"), ("http2", "boolean", "http2", "boolean"), ("tls", "boolean", "tls", "boolean"), ("tls_version", "string", "tls_version", "string"), ("tls_cipher", "string", "tls_cipher", "string"), ("response_status", "string", "response_status", "string"), ("response_text", "string", "response_text", "string"), ("response_bytes", "int", "response_bytes", "int"), ("response_cache", "string", "response_cache", "string"), ("cache_state", "string", "cache_state", "string"), ("cache_lastuse", "double", "cache_lastuse", "double"), ("cache_hits", "int", "cache_hits", "int"), ("server_region", "string", "server_region", "string"), ("server_datacenter", "string", "server_datacenter", "string")], transformation_ctx = "applymapping1")
## @type: DropNullFields
## @args: [transformation_ctx = "dropnullfields3"]
## @return: dropnullfields3
## @inputs: [frame = applymapping1]
dropnullfields3 = DropNullFields.apply(frame = applymapping1, transformation_ctx = "dropnullfields3")
## @type: Filter
## @args: [frame = dropnullfields3, f = filter_function, transformation_ctx = "filter4"]
## @return: filter4
## @inputs: [frame = dropnullfields3]
filter4 = Filter.apply(frame = dropnullfields3, f = filter_function, transformation_ctx = "filter4")
## @type: Map
## @args: [f = parse_user_agent, transformation_ctx = "map4"]
## @return: map4
## @inputs: [frame = filter4]
map4 = Map.apply(frame = filter4, f = map_function, transformation_ctx = "map4")
## @type: DataSink
## @args: [connection_type = "s3", connection_options = {"path": "s3://rubygems-logs.rubytogether/fastly_requests/"}, format = "parquet", transformation_ctx = "datasink4"]
## @return: datasink4
## @inputs: [frame = map4]
datasink4 = glueContext.write_dynamic_frame.from_options(frame = map4, connection_type = "s3", connection_options = {"path": "s3://rubygems-logs.rubytogether/fastly_requests/","partitionKeys":["year", "month", "day"]}, format = "parquet", transformation_ctx = "datasink4")
job.commit()