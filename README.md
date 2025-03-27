# kirby

Kirby slurps up the firehose of logs from Fastly and calculates daily counts for various Ruby ecosystem statistics, pretty quickly.

### How fast is pretty quickly?

For an 80MB gzipped log file containing 915,427 JSON event objects (which is 1.02GB uncompressed):

- 2.7 seconds total to read the entire file line by line
- 5.0 seconds total to also parse every JSON object into a Rust struct
- 7.8 seconds total to further parse every User Agent field for Bundler, RubyGems, and Ruby versions and other metrics

This is... very good. For comparison, a Python script that used AWS Glue to do something similar took about _30 minutes_. My first approach of writing a `nom` parser-combinator to parse the User Agent field, instead of using a regex, took 18.7 seconds. Processing a gigabyte of almost a million JSON objects into useful histograms in less than 8 seconds just blows my mind. But then I figured out how to use Rayon, and now it can parse 8 gzipped log files in parallel on an 8-core MacBook Pro, and that's super fast.

Then Rust got more optimized and Apple released the M1, and it got still faster. Finally, and I found the [profile-guided optimization](https://doc.rust-lang.org/rustc/profile-guided-optimization.html) docs, and it improved even more than I thought was still possible.

Most recently, it also turned out there was [a highly contended mutex around the regular expressions](https://github.com/rubytogether/kirby/pull/37) and that bought the multi-core version something like 40-60% more speed.

### Wait, _how_ fast?

          ~525 records/second/cpu in Python on Apache Spark in AWS Glue
       ~14,000 records/second/cpu in Ruby on a 2018 Intel MacBook Pro
      ~353,000 records/second/cpu in Rust on a 2018 Intel MacBook Pro
      ~550,000 records/second/cpu in Rust on a 2021 M1 MacBook Pro
      ~638,000 records/second/cpu in Rust on a 2021 M1 with PGO
      ~935,500 records/second/cpu in Rust on a 2025 M4 Max MacBook Pro
      ~983,500 records/second/cpu in Rust on a 2025 M4 Max with PGO
    ~1,240,000 records/second/cpu in Rust on a 2024 Ryzen 9 9950X with PGO

### Are you kidding me?

No. The latest version can parse records really, really fast.

         ~4,200 records/second in Python with 8 worker instances on AWS Glue
     ~1,085,000 records/second in Rust with rayon on an 8-core Intel MacBook Pro
     ~3,195,000 records/second in Rust with rayon on a 10-core M1 MacBook Pro
     ~3,583,000 records/second in Rust with rayon on M1 with PGO
    ~10,789,000 records/second in Rust with rayon on a 16-core M4 Max with PGO
    ~22,559,000 records/second in Rust with rayon on a 32-core Ryzen 9 9950X with PGO

### What does it calculate?

It counts Bundler, RubyGems, and Ruby versions, in hourly buckets, and prints those out as nested JSON to stdout.

### Tell me more about how this happened.

Okay, I wrote [a blog post with details about creating this library](https://andre.arko.net/2018/10/25/parsing-logs-230x-faster-with-rust/), and [a follow up about more optimizations](https://andre.arko.net/2019/01/11/parsing-logs-faster-with-rust-continued/).
