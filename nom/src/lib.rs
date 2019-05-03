#![feature(test)]

#[macro_use]
extern crate nom;
extern crate test;

use nom::character::complete::{digit1, multispace0, multispace1};
use std::str;

named!(int<&[u8], i32>, complete!(map_res!(digit1, |buf| unsafe{ str::from_utf8_unchecked(buf).parse::<i32>() })));

named!(sign<&[u8], bool>, do_parse!(
        sign: opt!(ws!(char!('-')))
        >> (sign.is_some())
));

named!(day<&[u8], Option<i32>>,
       opt!(do_parse!(
               day: int
               >> alt_complete!(
                   preceded!(multispace1, peek!(digit1))
                   | preceded!(multispace0, peek!(tag!(".")))
                   | preceded!(multispace0,eof!()))
               >> (day)
       ))
);

named!(hhmmss<&[u8], (Option<i32>, Option<i32>, Option<i32>)>, do_parse!(
        hour: opt!(int)
        >> has_mintue: map!(opt!(complete!(char!(':'))), |flag| flag.is_some())
        >> minute: cond_with_error!(has_mintue, int)
        >> has_second: map!(opt!(complete!(char!(':'))), |flag| flag.is_some())
        >> second: cond_with_error!(has_second, int)
        >> (hour, minute, second)
));

named!(
    fraction<Option<i32>>,
    opt!(complete!(do_parse!(
        char!('.') >> fraction: int >> (fraction)
    )))
);

named!(parse<&[u8],(bool,Option<i32>,Option<i32>,Option<i32>,Option<i32>,Option<i32>)>, do_parse!(
        sign: sign
        >> day: day
        >> hhmmss: hhmmss
        >> flag: map!(opt!(preceded!(multispace1, peek!(char!('.')))),
        |flag| flag.is_none() || hhmmss.0.is_none())
        >> fraction: cond_reduce!(flag, fraction)
        >> multispace0
        >> eof!()
        >> (sign, day, hhmmss.0, hhmmss.1, hhmmss.2, fraction)
));

#[bench]
fn bench_parse(b: &mut test::Bencher) {
    b.iter(|| {
        let time = test::black_box(b"-1 1:2:3.1234");
        let _ = test::black_box(parse(time));
    })
}
