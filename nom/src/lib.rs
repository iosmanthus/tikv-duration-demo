#![feature(test)]

extern crate test;

use bitfield::bitfield;
use nom::character::complete::{digit0, digit1, multispace0, multispace1};
use nom::{
    alt_complete, call, char, complete, cond_with_error, do_parse, eof, map, map_res, named,
    named_args, opt, peek, preceded, tag,
};
use std::time::Duration as StdDuration;
//use std::str;
type Result<T> = std::result::Result<T, ()>;

const TEN_POW: &[u32] = &[
    1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
];

pub const UNSPECIFIED_FSP: i8 = -1;
pub const MAX_FSP: i8 = 6;
pub const MIN_FSP: i8 = 0;
pub const DEFAULT_FSP: i8 = 0;
/*const MAX_DAYS: u32 = 34;*/
const MAX_HOURS: u32 = 838;
const MAX_MINUTES: u32 = 59;
const MAX_SECONDS: u32 = 59;

#[inline]
fn check_fsp(fsp: i8) -> Result<u8> {
    if fsp == UNSPECIFIED_FSP {
        return Ok(DEFAULT_FSP as u8);
    }
    if fsp > MAX_FSP || fsp < MIN_FSP {
        return Err(());
    }
    Ok(fsp as u8)
}

#[inline]
fn check_hour(hour: u32) -> Result<u32> {
    if hour > MAX_HOURS {
        Err(())
    } else {
        Ok(hour)
    }
}

#[inline]
fn check_minute(minute: u32) -> Result<u32> {
    if minute > MAX_MINUTES {
        Err(())
    } else {
        Ok(minute)
    }
}

#[inline]
fn check_second(second: u32) -> Result<u32> {
    if second > MAX_SECONDS {
        Err(())
    } else {
        Ok(second)
    }
}

fn buf_to_int(buf: &[u8]) -> u32 {
    buf.iter().fold(0, |acc, c| acc * 10 + (c - b'0') as u32)
}

named!(
    read_int<u32>,
    map_res!(digit1, |buf: &[u8]| if buf.len() > 7 {
        Err(())
    } else {
        Ok(buf_to_int(buf))
    })
);

named_args!(read_int_with_fsp(fsp: u8)<u32>, map_res!(
        digit1,
        |buf: &[u8]| -> Result<u32> {
            let fsp = fsp as usize;
            let (fraction, len) = if fsp >= buf.len() {
                (buf_to_int(buf), buf.len())
            } else {
                (buf_to_int(&buf[..=fsp]), fsp + 1)
            };
            Ok(fraction * TEN_POW[9 - len])
        }
));

named!(
    neg<bool>,
    do_parse!(
        neg: map!(opt!(complete!(char!('-'))), |flag| flag.is_some())
            >> cond_with_error!(
                neg,
                alt_complete!(preceded!(multispace1, preceded!(digit0, tag!("."))) | peek!(digit1))
            )
            >> (neg)
    )
);

named!(
    day<Option<u32>>,
    opt!(do_parse!(
        day: read_int
            >> alt_complete!(
                preceded!(multispace1, peek!(digit1))
                    | preceded!(multispace0, peek!(tag!(".")))
                    | preceded!(multispace0, eof!())
            )
            >> (day)
    ))
);

named!(
    hhmmss<(Option<u32>, Option<u32>, Option<u32>)>,
    do_parse!(
        hour: opt!(map_res!(read_int, check_hour))
            >> has_mintue: map!(opt!(complete!(char!(':'))), |flag| flag.is_some())
            >> minute: cond_with_error!(has_mintue, map_res!(read_int, check_minute))
            >> has_second: map!(opt!(complete!(char!(':'))), |flag| flag.is_some())
            >> second: cond_with_error!(has_second, map_res!(read_int, check_second))
            >> (hour, minute, second)
    )
);

named_args!(
    fraction(fsp: u8)<Option<u32>>,
    preceded!(opt!(complete!(char!('.'))), opt!(call!(read_int_with_fsp, fsp)))
);

named_args!(parse(fsp: u8)<
            (bool,          // neg
             Option<u32>,   // day
             Option<u32>,   // hour
             Option<u32>,   // minute
             Option<u32>,   // second
             Option<u32>)>, // fraction

            do_parse!(
                multispace0
                >> neg: neg
                >> day: day
                >> hhmmss: hhmmss
                >> fraction: call!(fraction, fsp)
                >> multispace0
                >> eof!()
                >> (neg, day, hhmmss.0, hhmmss.1, hhmmss.2, fraction)));

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Duration(u64);
    impl Debug;
    #[inline]
    bool, neg, set_neg: 55;
    #[inline]
    bool, unused, set_unused: 54;
    #[inline]
    u32, hour, set_hour: 53, 44;
    #[inline]
    u32, minute, set_minute: 43, 38;
    #[inline]
    u32, second, set_second: 37, 32;
    #[inline]
    u32, nano, set_nano: 31, 8;
    #[inline]
    pub u8, fsp, set_fsp: 7, 0;
}

impl Duration {
    pub fn parse(input: &[u8], fsp: i8) -> Result<Duration> {
        if input.is_empty() {
            return Err(());
        }
        let fsp = check_fsp(fsp)?;
        let (_, (neg, mut day, mut hour, mut minute, mut second, fraction)) =
            parse(input, fsp).map_err(|_| ())?;

        if day.is_some() && hour.is_none() {
            let block = day.take().unwrap();
            hour = Some(block / 10_000);
            minute = Some(block / 100 % 100);
            second = Some(block % 100);
        }

        let (hour, minute, second, fraction) = (
            hour.unwrap_or(0) + day.unwrap_or(0) * 24,
            minute.unwrap_or(0),
            second.unwrap_or(0),
            fraction.unwrap_or(0),
        );

        Duration::build(neg, hour, minute, second, fraction, fsp)
    }

    pub fn new(duration: StdDuration, neg: bool, fsp: i8) -> Result<Duration> {
        let fsp = check_fsp(fsp)?;

        let fraction = duration.subsec_nanos();
        let secs = duration.as_secs();

        let hour = secs / 3600;
        let minute = secs % 3600 / 60;
        let second = secs % 60;

        Duration::build(
            neg,
            hour as u32,
            minute as u32,
            second as u32,
            fraction,
            fsp,
        )
    }

    /// Build a `Duration` with details, truncate `fraction` with `fsp` and take the produced carry
    /// NOTE: the function assumes that the value of `hour/minute/second/fsp` is valid,
    /// so before you call function `build`, make sure you have checked their validity.
    fn build(
        neg: bool,
        mut hour: u32,
        mut minute: u32,
        mut second: u32,
        mut fraction: u32,
        fsp: u8,
    ) -> Result<Duration> {
        // Truncate `fraction` with `fsp`
        let mask = TEN_POW[(8 - fsp) as usize];
        fraction = ((fraction / mask + 5) / 10 * 10 * mask) / 1_000;

        if fraction >= 1_000_000 {
            fraction %= 1_000_000;
            second += 1;
            minute += second / 60;
            hour += minute / 60;
            second %= 60;
            minute %= 60;
            hour = check_hour(hour)?;
        }

        let mut duration = Duration(0);
        duration.set_neg(neg);
        duration.set_hour(hour);
        duration.set_minute(minute);
        duration.set_second(second);
        duration.set_nano(fraction);
        duration.set_fsp(fsp);
        Ok(duration)
    }
}

#[test]
fn it_works() {
    println!("{:#?}", Duration::parse(b"1:2:3.123", 6));
    println!("{:#?}", Duration::parse(b"1:2:3.1234567", 6));
    println!("{:#?}", Duration::parse(b"1:2:3.1234567", 4));
    println!("{:#?}", Duration::parse(b"1:2:3.123456", 4));
    println!(
        "{:#?}",
        Duration::new(StdDuration::new(3761, 123456789), false, 6)
    );
    println!("{:#?}", Duration::parse(b"1:59:59.99999", 4));
    println!("{:#?}", Duration::parse(b"- 1.12", 2));
    println!("{:#?}", Duration::parse(b"1 .12", 2));
    println!("{:#?}", Duration::parse(b"-1 .12", 2));
    println!("{:#?}", Duration::parse(b"-23", 2));
    println!("{:#?}", Duration::parse(b"  -1   1:2:3.99999  ", 2));

    println!("Should fail");
    println!("{:#?}", Duration::parse(b"- 1:1 .12", 2));
    println!("{:#?}", Duration::parse(b"- 1 .12", 2));
    println!("{:#?}", Duration::parse(b"-", 2));
    println!("{:#?}", Duration::parse(b"", 2));
}

#[bench]
fn bench_parse(b: &mut test::Bencher) {
    b.iter(|| {
        let time = test::black_box(b"  -1   1:2:3.99999  ");
        let _ = test::black_box(Duration::parse(time, test::black_box(3)));
    })
}
