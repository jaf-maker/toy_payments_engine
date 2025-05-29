# Toy Payment Engine

This repo contains a very simple payment engine. 

The engine was tested using 4 unit tests. Each test poins to a different input `transaction*.csv` file in the test_files folder. The results for the input files were calculated by hand and used as reference.

To ensure correctness, match flow control was used, as per the doc:
[https://doc.rust-lang.org/book/ch06-02-match.htmlxt](https://doc.rust-lang.org/book/ch06-02-match.html). Also extra steps were taken to handle when bad/incorrect data is parsed (Invalid row detection).

For efficiency `csv::ReaderBuilder` is used to parse the CSV input file, line by line. This avoids the need to load the entire file all at once.

For precision calculations `f64` types were preferred. No external floating point calculation crates are used. 

### Design Considerations

It is possible for the amount in an account to become negative if a dispute+chargeback come after a withdrawal. This is considered a plausible scenario since the account is locked afterwards.

All transaction ID's are assumed to be unique. No condition was created to handle the erroneous situation where more than one ID are the same.
