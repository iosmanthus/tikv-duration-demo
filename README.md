Usage:

```
cargo bench
```

It will output:

```
running 1 test
test tests::bench_issue_4025 ... bench:         932 ns/iter (+/- 132)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out

     Running target/release/deps/tikv_duration-e7224c92ff342413

running 1 test
test tests::bench_master ... bench:       1,624 ns/iter (+/- 413)

test result: ok. 0 passed; 0 failed; 0 ignored; 1 measured; 0 filtered out
```

