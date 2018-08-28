import sys
from awsglue.transforms import *
from awsglue.utils import getResolvedOptions
from pyspark.context import SparkContext
from awsglue.context import GlueContext
from awsglue.job import Job
from awsglue.dynamicframe import DynamicFrame

SOURCE_TABLE = "fastly_json"
DEST_TABLE = "fastly_stats"
DEST_FORMAT = "parquet"
SAMPLE_RATE = 4000
ROLLUP_COLUMNS = [
  "bundler",
  "ci",
  "platform",
  "ruby",
  "rubygems",
  "request_host",
  "tls_version"
]

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
  return random.randint(1, SAMPLE_RATE) == 1



## @params: [JOB_NAME]
args = getResolvedOptions(sys.argv, ['JOB_NAME'])

sc = SparkContext()
glueContext = GlueContext(sc)
spark = glueContext.spark_session
job = Job(glueContext)
job.init(args['JOB_NAME'], args)

#### JOB CONTENT START

fastly_json = glueContext.create_dynamic_frame.from_catalog(database = "rubygems", table_name = SOURCE_TABLE)
fastly_stats = fastly_json.select_fields(["timestamp", "user_agent", "request_host", "tls_version", "request_path"]).filter(filter_function).map(map_function)

from pyspark.sql.functions import lit
df = fastly_stats.toDF()
daily = spark.createDataFrame([], "struct<year:string,month:string,day:string,value:string,count:decimal,key:string>")
for name in ROLLUP_COLUMNS:
    count = df.groupBy("year", "month", "day", name).count()
    count = count.withColumnRenamed(name, "value").withColumn("key", lit(name)).dropna()
    daily = daily.union(count)

dl_count = df.filter("request_host = 'oregon.production.s3.rubygems.org.s3-us-west-2.amazonaws.com' AND request_path like '/gems/%'").groupBy("year", "month", "day").count()
dl_count = dl_count.withColumn("value", lit("all_gems")).withColumn("key", lit("downloads")).dropna().select("year", "month", "day", "value", "count", "key")
daily = daily.union(dl_count)

daily = daily.withColumn("count", daily["count"] * SAMPLE_RATE).repartition(1)
stats = DynamicFrame.fromDF(daily, glueContext, "fastly_stats")
glueContext.write_dynamic_frame.from_options(frame = stats, connection_type = "s3", connection_options = {"path": "s3://rubygems-logs.rubytogether/"+DEST_TABLE}, format = DEST_FORMAT)

#### JOB CONTENT END

job.commit()
