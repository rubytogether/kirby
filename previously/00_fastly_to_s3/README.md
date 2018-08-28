# RubyGems + Bundler ecosystem metrics

### What?

How to get ecosystem metrics from Fastly request logs:

1. Create an S3 bucket (in `us-east-1` or `us-west-2`, because that's where Athena is available) to hold the logs, create an IAM with ListBucket permission on the bucket  ARN, and GetObject and PutObject permission on "#{ARN}/\*".
1. Create a Fastly logging endpoint that sends logs to S3 using the bucket and IAM from the last step. You probably also want gzip level 9 and to use the "infrequent access" S3 tier, that'll make this a lot cheaper. Add a condition to this logging endpoint like `status == 1000`. We just need to ensure it will never trigger on its own.
1. Install [vcl-json-generate](https://github.com/fastly/vcl-json-generate) in your Fastly config, uploading the custom VCL file and adding an include line at the top of your existing `main.vcl`.
1. Add a VCL snippet to the `log` section with the contents of [json\_request\_logs.vcl](./json_request_logs.vcl), replacing the last line with the correct `syslog` instruction to send the logs to your new logging endpoint. The logs sent to S3 will contain data like the sample named [fastly\_json.log](./sample_data/fastly_json.log).
1. Over in [Amazon Athena](https://us-west-2.console.aws.amazon.com/athena/home?region=us-west-2), run the query from [athena\_create\_table.sql](./athena_create_table.sql), changing the URL to point to your bucket.
1. Run any arbitrary SQL query against the JSON you now have living in S3 using Athena. Pretty neat.
1. Extract the user agent strings from the overall logs by running [athena\_select\_user\_agent.sql](./athena_select_user_agent.sql) and downloading the result. The output from the query will look like the sample named [user\_agents.csv][./sample_data/user_agents.csv].
1. Feed the file of user agents to [parse\_user\_agents.rb](./parse_user_agents.rb) to get some stats on what Bundler, RubyGems, and Ruby versions created the logs you queried. The output will look like the sample named [user\_agent\_counts.json](./sample_data/user_agent_counts.json).

### Ideas

- Query for and parse user agents that are only RubyGems (they start `Ruby, RubyGems`)
- Add timestamp to the user agent list to make it possible to generate time series graphs
- Create a Redshift database to store the time series user agent data
- Roll up data into hourly or maybe daily summaries, and store those in Redshift instead
- Set up a QuickLook instance that graphs the time series data from either Athena or Redshift
- Create a Lambda task that rewrites the data in S3 to partition it
- Create an EWR or Lambda task that rewrites the data in S3 into Adobe Parquet columnar form to massively reduce storage cost and query time/cost

### Further reading

- [Analyzing VPC Flow Logs with Amazon Kinesis Firehose, Amazon Athena, and Amazon QuickSight](https://aws.amazon.com/blogs/big-data/analyzing-vpc-flow-logs-with-amazon-kinesis-firehose-amazon-athena-and-amazon-quicksight/)  
  Contains examples of: Athena from S3, QuickSight from Athena, Lambda to partition S3 data for Athena
- [Analyzing Data in S3 using Amazon Athena](https://aws.amazon.com/blogs/big-data/analyzing-data-in-s3-using-amazon-athena/)  
  Contains examples of: Athena from S3, Using partitioned data in Athena, EWR job to convert data to parquet
