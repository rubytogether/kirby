# kirby

Kirby slurps up the firehose of logs from Fastly and calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects (which is 1.02GB uncompressed):

- 2.7 seconds total to read the entire file line by line
- 5.0 seconds total to also parse every JSON object into a Rust struct
- 7.8 seconds total to further parse every User Agent field for Bundler, RubyGems, and Ruby versions and other metrics

This is... very good. For comparison, a Python script that used AWS Glue to do something similar took about _30 minutes_. My first approach of writing a `nom` parser-combinator to parse the User Agent field, instead of using a regex, took 18.7 seconds. Processing a gigabyte of almost a million JSON objects into useful histograms in less than 8 seconds just blows my mind. But then I figured out how to use Rayon, and now it can parse 8 gzipped log files in parallel on an 8-core MacBook Pro, and that's super fast.

### Wait, _how_ fast?

        ~525 records/second/cpu in Python on AWS Glue
    ~300,000 records/second/cpu in Rust with regex

### Are you kidding me?

No. The latest version (which I am now benchmarking without also running `cargo build` 🤦🏻‍♂️) can parse records really, really fast.

        ~4,200 records/second in Python with 8 worker instances on AWS Glue
    ~1,085,000 records/second in Rust with 8 cores and rayon on a MacBook Pro

### What does it calculate?

It counts Bundler, RubyGems, and Ruby versions, in hourly buckets, and prints those out as nested JSON to stdout.

### Tell me more about how this happened.

Okay, I wrote [a blog post with details about creating this library](https://andre.arko.net/2018/10/25/parsing-logs-230x-faster-with-rust/).
