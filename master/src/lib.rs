#![feature(test)]
extern crate test;
use std::str;
use time::{self, Tm};

pub const UNSPECIFIED_FSP: i8 = -1;
pub const MAX_FSP: i8 = 6;
pub const MIN_FSP: i8 = 0;
pub const DEFAULT_FSP: i8 = 0;
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
pub const NANO_WIDTH: u32 = 9;
const SECS_PER_HOUR: u64 = 3600;
const MAX_TIME_IN_SECS: u64 = 838 * SECS_PER_HOUR + 59 * SECS_PER_MINUTE + 59;
const SECS_PER_MINUTE: u64 = 60;

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Debug, Clone, Copy)]
pub struct Duration {
    dur: StdDuration,
    fsp: u8,
    neg: bool,
}

fn tm_to_secs(t: Tm) -> u64 {
    t.tm_hour as u64 * SECS_PER_HOUR + t.tm_min as u64 * SECS_PER_MINUTE + t.tm_sec as u64
}

use std::time::Duration as StdDuration;

fn check_dur(dur: &StdDuration) -> Result<()> {
    let secs = dur.as_secs();
    if secs > MAX_TIME_IN_SECS || secs == MAX_TIME_IN_SECS && dur.subsec_nanos() > 0 {
        return Err("invalid time");
    }
    Ok(())
}

fn parse_frac(s: &[u8], fsp: u8) -> Result<u32> {
    if s.is_empty() {
        return Ok(0);
    }

    if s.iter().any(|&c| c < b'0' || c > b'9') {
        return Err("Error");
    }
    let res = s
        .iter()
        .take(fsp as usize + 1)
        .fold(0, |l, r| l * 10 + u32::from(r - b'0'));
    if s.len() > fsp as usize {
        if res % 10 >= 5 {
            Ok(res / 10 + 1)
        } else {
            Ok(res / 10)
        }
    } else {
        Ok(res * 10u32.pow((fsp as usize - s.len()) as u32))
    }
}

fn check_fsp(fsp: i8) -> Result<u8> {
    if fsp == UNSPECIFIED_FSP {
        return Ok(DEFAULT_FSP as u8);
    }
    if fsp > MAX_FSP || fsp < MIN_FSP {
        return Err("Invalid fsp");
    }
    Ok(fsp as u8)
}

impl Duration {
    pub fn new(dur: StdDuration, neg: bool, fsp: i8) -> Result<Duration> {
        check_dur(&dur)?;
        Ok(Duration {
            dur,
            neg,
            fsp: check_fsp(fsp)?,
        })
    }

    #[inline]
    pub fn zero() -> Duration {
        Duration {
            dur: StdDuration::from_secs(0),
            neg: false,
            fsp: 0,
        }
    }

    pub fn parse(mut s: &[u8], fsp: i8) -> Result<Duration> {
        let fsp = check_fsp(fsp)?;

        let (mut neg, mut day, mut frac) = (false, None, 0);

        if s.is_empty() {
            return Ok(Duration::zero());
        } else if s[0] == b'-' {
            s = &s[1..];
            neg = true;
        }

        let mut parts = s.splitn(2, |c| *c == b' ');
        s = parts.next().unwrap();
        if let Some(remain) = parts.next() {
            let day_str = str::from_utf8(s).map_err(|_| "fail to parse day")?;
            day = Some(u64::from_str_radix(day_str, 10).map_err(|_| "fail to parse day")?);
            s = remain;
        }

        let mut parts = s.splitn(2, |c| *c == b'.');
        s = parts.next().unwrap();
        if let Some(frac_part) = parts.next() {
            frac = parse_frac(frac_part, fsp)?;
            frac *= 10u32.pow(NANO_WIDTH - u32::from(fsp));
        }

        let mut parts = s.splitn(2, |c| *c == b':');
        s = parts.next().unwrap();
        let s_str = str::from_utf8(s).map_err(|_| "")?;
        let mut secs;
        match parts.next() {
            Some(remain) => {
                let remain_str = str::from_utf8(remain).map_err(|_| "")?;
                let t = match remain.len() {
                    5 => time::strptime(remain_str, "%M:%S"),
                    2 => time::strptime(remain_str, "%M"),
                    _ => return Err(""),
                }
                .map_err(|_| "")?;
                secs = tm_to_secs(t);
                secs += u64::from_str_radix(s_str, 10).map_err(|_| "")? * SECS_PER_HOUR;
            }
            None if day.is_some() => {
                secs = u64::from_str_radix(s_str, 10).map_err(|_| "")? * SECS_PER_HOUR;
            }
            None => {
                let t = match s.len() {
                    6 => time::strptime(s_str, "%H%M%S"),
                    4 => time::strptime(s_str, "%M%S"),
                    2 => time::strptime(s_str, "%S"),
                    _ => return Err(""),
                }
                .map_err(|_| "")?;
                secs = tm_to_secs(t);
            }
        }

        if let Some(day) = day {
            secs += day * SECS_PER_HOUR * 24;
        }

        let dur = StdDuration::new(secs, frac);
        Duration::new(dur, neg, fsp as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[bench]
    fn bench_parse(b: &mut test::Bencher) {
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
