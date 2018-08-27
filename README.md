# ecostats

Calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects:

- *3.6 seconds* to read the entire file line by line
- *27.4 seconds* to parse every JSON object into a Rust struct
- *63 seconds*  to further parse every User Agent field for version information

This is... very good. For comparison, just switching the User Agent parsing to `regex` from a `nom` parser-combinator increased the time for the same result to around 3.5 minutes.

### What does it calculate?

Some... stuff. Working on it.