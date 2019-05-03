Usage:

```
cargo bench
```

It will output:

```
     Running target/release/deps/dfa-b32397db91631f85

running 2 tests
test dbg_parse ... ignored
test bench ... bench:          39 ns/iter (+/- 7)

test result: ok. 0 passed; 0 failed; 1 ignored; 1 measured; 0 filtered out

     Running target/release/deps/nom-866c42ecd6019f4a

running 1 test
test bench_parse ... bench:          41 ns/iter (+/- 5)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/pull_4427-690706fcc3b31bfb

running 1 test
test tests::bench_issue_4025 ... bench:         373 ns/iter (+/- 34)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/regex-e52ec04290537e86

running 1 test
test bench_regex ... bench:         469 ns/iter (+/- 142)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/tikv_duration-66098b0818d1ecfb

running 1 test
test tests::bench_master ... bench:         747 ns/iter (+/- 162)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

```
