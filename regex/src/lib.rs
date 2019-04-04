#![feature(test)]
extern crate test;

#[bench]
fn bench_regex(b: &mut test::Bencher) {
    use regex::Regex;
    let re = Regex::new(r"\s*(\d*)\s*(\d*):?(\d*):?(\d*)\s*").unwrap();
    b.iter(|| {
        let re = test::black_box(&re);
        test::black_box(re.captures("1 23:45:37"));
    })
}
