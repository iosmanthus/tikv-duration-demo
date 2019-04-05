#![feature(test)]
extern crate test;
use bitfield::bitfield;

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

    pub fn parse(s: &[u8], fsp: i8) -> Result<Duration> {
        use State::*;
        #[derive(PartialEq, Debug)]
        enum State {
            Start,
            Block,
            PostBlock,
            Hour,
            MinuteColon,
            Minute,
            SecondColon,
            Second,
            Dot,
            Fraction,
            Consume,
            End,
        }

        let fsp = check_fsp(fsp)?;
        let check_block = |block| if block > 8385959 { Err(()) } else { Ok(block) };
        let to_dec = |d| u64::from(d - b'0');

        let mut neg = false;
        let (mut block, mut day, mut hour, mut minute, mut second, mut fract) = (0, 0, 0, 0, 0, 0);
        let mut eaten = 1;

        let mut state = Start;
        for &c in s {
            if c == b'.'
                && (state == Start
                    || state == Block
                    || state == PostBlock
                    || state == Hour
                    || state == Minute
                    || state == Second)
            {
                state = Dot;
                continue;
            }
            state = match state {
                Start => {
                    if c.is_ascii_digit() {
                        block = to_dec(c);
                        Block
                    } else if c.is_ascii_whitespace() {
                        Start
                    } else if c == b'-' {
                        if neg {
                            return Err(());
                        } else {
                            neg = true;
                            Start
                        }
                    } else {
                        return Err(());
                    }
                }
                Block => {
                    if c.is_ascii_digit() {
                        block = check_block(block * 10 + to_dec(c))?;
                        Block
                    } else if c.is_ascii_whitespace() {
                        PostBlock
                    } else if c == b':' {
                        hour = block;
                        block = 0;
                        MinuteColon
                    } else {
                        return Err(());
                    }
                }
                PostBlock => {
                    if c.is_ascii_digit() {
                        hour = to_dec(c);
                        day = block;
                        block = 0;
                        Hour
                    } else if c.is_ascii_whitespace() {
                        PostBlock
                    } else {
                        return Err(());
                    }
                }
                Hour => {
                    if c.is_ascii_digit() {
                        hour = check_hour(hour * 10 + to_dec(c))?;
                        Hour
                    } else if c.is_ascii_whitespace() {
                        End
                    } else if c == b':' {
                        MinuteColon
                    } else {
                        return Err(());
                    }
                }
                MinuteColon => {
                    if c.is_ascii_digit() {
                        minute = to_dec(c);
                        Minute
                    } else {
                        return Err(());
                    }
                }
                Minute => {
                    if c.is_ascii_digit() {
                        minute = check_minute(minute * 10 + to_dec(c))?;
                        Minute
                    } else if c.is_ascii_whitespace() {
                        End
                    } else if c == b':' {
                        SecondColon
                    } else {
                        return Err(());
                    }
                }
                SecondColon => {
                    if c.is_ascii_digit() {
                        second = to_dec(c);
                        Second
                    } else {
                        return Err(());
                    }
                }
                Second => {
                    if c.is_ascii_digit() {
                        second = check_second(second * 10 + to_dec(c))?;
                        Second
                    } else if c.is_ascii_whitespace() {
                        End
                    } else {
                        return Err(());
                    }
                }
                Dot => {
                    if c.is_ascii_digit() {
                        fract = to_dec(c);
                        Fraction
                    } else {
                        return Err(());
                    }
                }
                Fraction => {
                    if c.is_ascii_digit() {
                        if eaten < fsp {
                            fract = fract * 10 + to_dec(c);
                            eaten += 1;
                            Fraction
                        } else {
                            if to_dec(c) > 4 {
                                fract += 1;
                            }
                            Consume
                        }
                    } else if c.is_ascii_whitespace() {
                        End
                    } else {
                        return Err(());
                    }
                }
                Consume => {
                    if c.is_ascii_digit() {
                        Consume
                    } else if c.is_ascii_whitespace() {
                        End
                    } else {
                        return Err(());
                    }
                }
                End => {
                    if c.is_ascii_whitespace() {
                        End
                    } else {
                        return Err(());
                    }
                }
            };
        }
        if state == MinuteColon || state == SecondColon {
            return Err(());
        }
        hour += day * 24;
        fract *= u64::from(TEN_POW[NANO_WIDTH as usize - fsp as usize]);
        Duration::with_detail(neg, hour, minute, second, fract, fsp)
    }
}

#[test]
fn dbg_parse() {
    match Duration::parse(b"10:10:1\xff", 0) {
        Ok(duration) => {
            dbg!(duration);
        }
        Err(_) => {
            dbg!("error");
        }
    };
}

#[bench]
fn bench(b: &mut test::Bencher) {
    b.iter(|| {
        let _ = test::black_box(Duration::parse(test::black_box(b"-1 1:2:3.123567"), 6));
    })
}
