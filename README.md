# kirby

Kirby slurps up the firehose of logs from Fastly and calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects (which is 1.02GB uncompressed):

- **3.6 seconds** to read the entire file line by line
- **27.4 seconds** to parse every JSON object into a Rust struct
- **63 seconds**  to further parse every User Agent field for version information

This is... very good. For comparison, a Python script that used AWS Glue to do something similar took about 30 minutes. Just switching the User Agent parsing to `regex` from a `nom` parser-combinator increased the time for the same result to around 3.5 minutes.

### What does it calculate?

Some... stuff. Working on it.
