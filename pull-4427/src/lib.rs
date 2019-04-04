#![feature(test)]
extern crate test;
use bitfield::bitfield;
use std::str;
const TEN_POW: &[u32] = &[
    1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
];

pub const UNSPECIFIED_FSP: i8 = -1;
pub const MAX_FSP: i8 = 6;
pub const MIN_FSP: i8 = 0;
pub const DEFAULT_FSP: i8 = 0;
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
pub const NANO_WIDTH: u32 = 9;
const MAX_HOURS: u64 = 838;
const MAX_MINUTES: u64 = 59;
const MAX_SECONDS: u64 = 59;
const MINUTES_PER_HOUR: u64 = 60;
const SECS_PER_MINUTE: u64 = 60;

type Result<T> = std::result::Result<T, ()>;
bitfield! {
    #[derive(Clone, Copy)]
    pub struct Duration(u64);
    impl Debug;
    #[inline]
    bool, neg, set_neg: 63;
    #[inline]
    u64, hour, set_hour: 62, 53;
    #[inline]
    u64, minute, set_minute: 52, 47;
    #[inline]
    u64, second, set_second: 46, 41;
    #[inline]
    u64, nano, set_nano: 40, 9;
    #[inline]
    pub u8, fsp, set_fsp: 8, 1;
    #[inline]
    bool, unused, set_unused: 0;
}

#[inline]
fn check_hour(hour: u64) -> Result<u64> {
    if hour > MAX_HOURS {
        Err(())
    } else {
        Ok(hour)
    }
}

#[inline]
fn check_minute(minute: u64) -> Result<u64> {
    if minute > MAX_MINUTES {
        Err(())
    } else {
        Ok(minute)
    }
}

#[inline]
fn check_second(second: u64) -> Result<u64> {
    if second > MAX_SECONDS {
        Err(())
    } else {
        Ok(second)
    }
}

fn check_fsp(fsp: i8) -> Result<u8> {
    if fsp == UNSPECIFIED_FSP {
        return Ok(DEFAULT_FSP as u8);
    }
    if fsp > MAX_FSP || fsp < MIN_FSP {
        return Err(());
    }
    Ok(fsp as u8)
}

fn parse_frac(frac: &str, fsp: u8) -> Result<u64> {
    if frac.is_empty() {
        return Ok(0);
    }

    let fsp = fsp as usize;
    let mapping = |_| ();

    Ok(if frac.len() <= fsp {
        frac.parse::<u64>().map_err(mapping)? * u64::from(TEN_POW[fsp - frac.len()])
    } else {
        let result = frac[..=fsp].parse::<u64>().map_err(mapping)?;
        if result % 10 > 4 {
            result / 10 + 1
        } else {
            result / 10
        }
    })
}

impl Duration {
    #[inline]
    pub fn zero() -> Self {
        Duration(0)
    }
    #[inline]
    fn with_detail(
        neg: bool,
        mut hour: u64,
        mut minute: u64,
        mut second: u64,
        mut nano: u64,
        fsp: u8,
    ) -> Result<Duration> {
        let round = u64::from(TEN_POW[NANO_WIDTH as usize - fsp as usize - 1]);
        if nano / round % 10 > 4 {
            let padding = round * 10;
            nano = (nano / padding + 1) * padding;
        }

        second += nano / NANOS_PER_SEC;
        minute += second / SECS_PER_MINUTE;
        hour += minute / MINUTES_PER_HOUR;
        check_hour(hour)?;

        nano %= NANOS_PER_SEC;
        second %= SECS_PER_MINUTE;
        minute %= MINUTES_PER_HOUR;

        let mut duration = Duration(0);
        duration.set_neg(neg);
        duration.set_hour(hour);
        duration.set_minute(minute);
        duration.set_second(second);
        duration.set_nano(nano);
        duration.set_fsp(fsp);
        Ok(duration)
    }

    pub fn parse(mut s: &[u8], fsp: i8) -> Result<Duration> {
        let fsp = check_fsp(fsp)?;

        if s.is_empty() {
            let mut zero = Duration::zero();
            zero.set_fsp(fsp);
            return Ok(zero);
        }

        let neg = if s[0] == b'-' {
            s = &s[1..];
            true
        } else {
            false
        };

        let mut day = None;
        let mut parts = s.splitn(2, |c| *c == b' ');
        s = parts.next().unwrap();
        if let Some(part) = parts.next() {
            day = Some(
                unsafe { str::from_utf8_unchecked(s) }
                    .parse::<u64>()
                    .map_err(|_| ())?,
            );
            s = part;
        }

        let mut nano = 0;
        let mut parts = s.splitn(2, |c| *c == b'.');
        s = parts.next().unwrap();
        if let Some(frac) = parts.next() {
            nano = parse_frac(unsafe { str::from_utf8_unchecked(frac) }, fsp)?
                * u64::from(TEN_POW[NANO_WIDTH as usize - fsp as usize]);
        }
        let mut parts = s.splitn(3, |c| *c == b':');
        let first = parts.next().ok_or(())?;

        let first_try = unsafe { str::from_utf8_unchecked(first) }.parse::<u64>();
        let mut hour;
        let (mut minute, mut second) = (0, 0);
        match parts.next() {
            Some(part) => {
                hour = first_try.map_err(|_| ())?;
                minute = unsafe { str::from_utf8_unchecked(part) }
                    .parse::<u64>()
                    .map_err(|_| ())
                    .and_then(check_minute)?;

                if let Some(part) = parts.next() {
                    second = unsafe { str::from_utf8_unchecked(part) }
                        .parse::<u64>()
                        .map_err(|_| ())
                        .and_then(check_second)?;
                }
            }
            None if day.is_some() => {
                hour = first_try.map_err(|_| ())?;
            }
            None => {
                let time = first_try.map_err(|_| ())?;
                second = check_second(time % 100)?;
                minute = check_minute(time / 100 % 100)?;
                hour = time / 1_00_00;
            }
        }
        hour += day.unwrap_or(0) * 24;
        Duration::with_detail(neg, hour, minute, second, nano, fsp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[bench]
    fn bench_issue_4025(b: &mut test::Bencher) {
        let cases = vec![
            ("12:34:56.1234", 0),
            ("12:34:56.789", 1),
            ("10:20:30.189", 2),
            ("2 27:54:32.828", 3),
            ("2 33:44:55.666777", 4),
            ("112233.445566", 5),
            ("1 23", 5),
            ("1 23:12.1234567", 6),
        ];
        b.iter(|| {
            let cases = test::black_box(&cases);
            for &(s, fsp) in cases {
                let _ = test::black_box(Duration::parse(s.as_bytes(), fsp).unwrap());
            }
        })
    }

}
