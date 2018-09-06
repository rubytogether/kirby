# kirby

Kirby slurps up the firehose of logs from Fastly and calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects (which is 1.02GB uncompressed):

- **2.7 seconds total** to read the entire file line by line
- **5.0 seconds total** to also parse every JSON object into a Rust struct
- **7.8 seconds total**  to further parse every User Agent field for Bundler, RubyGems, and Ruby versions and other metrics

This is... very good. For comparison, a Python script that used AWS Glue to do something similar took about _30 minutes_. My first approach of writing a `nom` parser-combinator to parse the User Agent field, instead of using a regex, took 18.7 seconds. Processing a gigabyte of almost a million JSON objects into useful histograms in less than 8 seconds just blows my mind.

### What does it calculate?

It counts Bundler, RubyGems, and Ruby versions and prints those out as a nested hash to stdout.