# kirby

Kirby slurps up the firehose of logs from Fastly and calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects (which is 1.02GB uncompressed):

- 2.7 seconds total to read the entire file line by line
- 5.0 seconds total to also parse every JSON object into a Rust struct
- 7.8 seconds total to further parse every User Agent field for Bundler, RubyGems, and Ruby versions and other metrics

This is... very good. For comparison, a Python script that used AWS Glue to do something similar took about _30 minutes_. My first approach of writing a `nom` parser-combinator to parse the User Agent field, instead of using a regex, took 18.7 seconds. Processing a gigabyte of almost a million JSON objects into useful histograms in less than 8 seconds just blows my mind. But then I figured out how to use Rayon, and now if you give it 8 gzipped log files on an 8-core MacBook Pro, it can parse 399,300 JSON objects per second.

### Wait, _how_ fast?

       ~525 records/second/cpu in Python on AWS Glue
     50,534 records/second/cpu in Rust with nom
    121,153 records/second/cpu in Rust with regex

### Are you kidding me?

No. It gets even better if you have multiple cores.

     ~4,200 records/second in Python with 8 worker instances on AWS Glue
    399,300 records/second in Rust with 8 cores and rayon on a MacBook Pro

### What does it calculate?

It counts Bundler, RubyGems, and Ruby versions, in hourly buckets, and prints those out as nested JSON to stdout.