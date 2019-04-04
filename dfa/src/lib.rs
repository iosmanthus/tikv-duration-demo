#![feature(test)]
extern crate test;
#[derive(Debug)]
pub struct Time {
    neg: bool,
    hour: u64,
    minute: u64,
    second: u64,
    frac: u64,
}
pub fn parse(string: &[u8], fsp: i8) -> Time {
    // TODO: check fsp
    #[derive(PartialEq)]
    enum State {
        Start,
        Block,
        Mid,
        Hour,
        Minute,
        Second,
        Fraction,
        End,
    }
    use State::*;
    let mut state = Start;
    let (mut neg, mut block, mut day, mut hour, mut minute, mut second, mut frac) =
        (false, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let mut take = 0;
    for (i, &c) in string.iter().enumerate() {
        if c == b'.' && state != Start {
            if i == string.len() {
                panic!();
            }
            state = Fraction;
            continue;
        } else if c == b' '
            && (state == Hour || state == Minute || state == Second || state == Fraction)
        {
            state = End;
            continue;
        }
        match state {
            Start => {
                if c.is_ascii_digit() {
                    state = Block;
                    block = u64::from(c - b'0');
                } else if c.is_ascii_whitespace() {
                    continue;
                } else if c == b'-' {
                    if i == string.len() {
                        panic!();
                    }
                    neg = true;
                    continue;
                } else {
                    panic!();
                };
            }
            Block => {
                if c.is_ascii_digit() {
                    // TODO: Handling overflow
                    block = block * 10 + u64::from(c - b'0');
                } else if c.is_ascii_whitespace() {
                    state = Mid;
                    continue;
                } else if c == b':' {
                    if i == string.len() {
                        panic!();
                    }
                    state = Minute;
                    hour = block;
                    block = 0;
                } else {
                    panic!();
                }
            }
            Mid => {
                if c.is_ascii_digit() {
                    state = Hour;
                    day = block;
                    hour = u64::from(c - b'0');
                    block = 0;
                    continue;
                } else if c.is_ascii_whitespace() {
                    continue;
                } else {
                    panic!();
                }
            }
            Hour => {
                if c.is_ascii_digit() {
                    // TODO: Check hour
                    hour = hour * 10 + u64::from(c - b'0');
                } else if c == b':' {
                    if i == string.len() {
                        panic!();
                    }
                    state = Minute;
                    continue;
                } else {
                    panic!();
                }
            }
            Minute => {
                if c.is_ascii_digit() {
                    // TODO: Check minute
                    minute = minute * 10 + u64::from(c - b'0');
                } else if c == b':' {
                    if i == string.len() {
                        panic!();
                    }
                    state = Second;
                    continue;
                } else {
                    panic!();
                }
            }
            Second => {
                if c.is_ascii_digit() {
                    // TODO: Check second
                    second = second * 10 + u64::from(c - b'0');
                } else if c == b':' {
                    if i == string.len() {
                        panic!();
                    }
                    continue;
                } else {
                    panic!();
                }
            }
            Fraction => {
                if c.is_ascii_digit() {
                    take += 1;
                    frac = frac * 10 + u64::from(c - b'0');
                    if take == fsp {
                        state = End;
                        continue;
                    }
                } else {
                    panic!();
                }
            }
            End => {
                if !c.is_ascii_whitespace() {
                    panic!();
                }
            }
        }
    }
    if state == Block || state == Mid {
        second = block % 100;
        minute = block / 100 % 100;
        hour = block / 100_00 % 100;
    }
    hour += day * 24;
    Time {
        neg,
        hour,
        minute,
        second,
        frac,
    }
}

#[bench]
fn bench(b: &mut test::Bencher) {
    b.iter(|| {
        let _ = test::black_box(parse(test::black_box(b"-1:2:3.123567"), 6));
    })
}
