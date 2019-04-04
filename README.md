Usage:

```
cargo bench
```

It will output:

```
running 1 test
test bench ... bench:          92 ns/iter (+/- 8)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/pull_4427-db77f207f2d94e38

running 1 test
test tests::bench_issue_4025 ... bench:         821 ns/iter (+/- 567)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/regex-0579d0acc8b7599c

running 1 test
test bench_regex ... bench:       1,077 ns/iter (+/- 202)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/tikv_duration-e7224c92ff342413

running 1 test
test tests::bench_master ... bench:       1,457 ns/iter (+/- 449)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out
```

