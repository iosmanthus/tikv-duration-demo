Usage:

```
cargo bench
```

It will output:

```
    Finished release [optimized] target(s) in 0.03s
     Running target/release/deps/dfa-e1a1d3bbe7207791

running 1 test
test bench ... bench:          93 ns/iter (+/- 62)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/pull_4427-db77f207f2d94e38

running 1 test
test tests::bench_issue_4025 ... bench:         975 ns/iter (+/- 194)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/regex-0579d0acc8b7599c

running 1 test
test bench_regex ... bench:       1,343 ns/iter (+/- 182)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/tikv_duration-e7224c92ff342413

running 1 test
test tests::bench_master ... bench:       1,642 ns/iter (+/- 194)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

```

