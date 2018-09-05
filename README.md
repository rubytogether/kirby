# kirby

Kirby slurps up the firehose of logs from Fastly and calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects (which is 1.02GB uncompressed):

- **3.6 seconds total** to read the entire file line by line
- **27.4 seconds total** to also parse every JSON object into a Rust struct
- **63 seconds total**  to further parse every User Agent field for versions and metrics from Bundler, RubyGems, Ruby, and more

This is... very good. For comparison, a Python script that used AWS Glue to do something similar took about 30 minutes. Writing a `nom` parser-combinator to parse the User Agent field instead of using a regex increased the time for the same result to around 3.5 minutes. It's pretty fast!

### What does it calculate?

Some... stuff. Working on it.
