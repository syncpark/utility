use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use regex::Regex;
use std::convert::TryFrom;
use std::str;

fn main() {
    let input = 1_562_050_801_302_957_000_u64;
    let start = from_u64(input);
    println!("{input} => {start}");
    println!("{}", parse_rfc3339_weak(&start).format("%FT%T%.9fZ"));
}

const A_BILLION: i64 = 1_000_000_000;
fn from_u64(input: u64) -> String {
    let input = i64::try_from(input).unwrap_or_default();
    let nsecs = u32::try_from(input % A_BILLION).unwrap_or_default();
    NaiveDateTime::from_timestamp_opt(input / A_BILLION, nsecs)
        .map_or("-".to_string(), |s| s.format("%FT%T%.9fZ").to_string())
}

fn parse_rfc3339_weak(s: &str) -> DateTime<Utc> {
    chrono::DateTime::from(humantime::parse_rfc3339_weak(s).unwrap())
}

#[allow(unused)]
fn test_code() {
    let value = "1614501549.708923";
    println!("input={value}");
    if value.find('.').is_some() {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%s%.6f") {
            let ts: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            println!("dot = {ts:?}");
        }
    } else if let Ok(secs) = value.parse::<i64>() {
        let ts: DateTime<Utc> = Utc.timestamp(secs, 0);
        println!("no-dot = {ts:?}");
    }
}

#[allow(unused)]
fn from_utc_to_local(v: &str) {
    if let Ok(naive) = NaiveDateTime::parse_from_str(v, "%s%.6f") {
        let ts: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        println!("input = {v}");
        println!("\tUtc = {}", ts.format("%Y-%m-%d %H:%M:%S"));
        println!(
            "\tLocal {}",
            DateTime::<Local>::from(ts).format("%Y-%m-%d %H:%M:%S")
        );
    }
}

#[allow(unused)]
fn from_utc() {
    let now = Utc::now();
    let secs = now.timestamp();
    let nanos = now.timestamp_nanos();
    println!("now = {now:?}, timestamp = {secs}, timestamp_nanos = {nanos}");
    println!(
        "timestamp_nanos - timestamp * 1000000000 = {}",
        nanos - secs * 1_000_000_000
    );
}

#[allow(unused)]
fn from_i64() {
    let i = 1_637_220_759_i64;
    let naive = NaiveDateTime::from_timestamp(i, 0);

    // Create a normal DateTime from the NaiveDateTime
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    println!("i = {i}, datetime = {datetime:?}");
}

#[allow(unused)]
fn from_i64_with_nanos(v: &str) {
    //let v = "1562049953.716246";
    println!("value = {v}");
    let s: Vec<_> = v.split('.').map(String::from).collect();
    println!("{s:?}");
    let secs = s
        .get(0)
        .map_or(0, |ss| ss.parse::<i64>().unwrap_or_default());
    let nanos = s
        .get(1)
        .map_or(0, |ss| ss.parse::<u32>().unwrap_or_default() * 1000);
    let dt_no_nano = Utc.timestamp(secs, 0);
    let dt_with_nano = Utc.timestamp(secs, nanos);
    println!("dt_no_nano = {dt_no_nano:?}, dt_with_nano = {dt_with_nano:?}");

    if let Ok(naive) = NaiveDateTime::parse_from_str(v, "%s%.6f") {
        println!(
            "naive = {naive:?}\ni64 = {}\ntimestamp_nanos = {}\ntimestamp_nanos = {}",
            naive.timestamp(),
            naive.timestamp_nanos(),
            naive.format("%s%.6f")
        );
    }
}

#[allow(unused)]
fn test_as_bytes(tt: &[u8]) {
    if let Ok(s) = str::from_utf8(tt) {
        println!("Ok: from_utf8 = {s}");
    } else {
        println!("Err: from_utf8");
    }
}

#[allow(unused)]
fn parse_dt(s: &str) -> i64 {
    // / 18/Apr/2019:16:22:00 +0900 => 2019 Apr 18 16:22:00 +0900
    if let Ok(re) = Regex::new(
        r"(?P<day>\d+)/(?P<month>[a-zA-Z]+)/(?P<year>\d{4}):(?P<hour>\d{2}):(?P<min>\d{2}):(?P<sec>\d{2}) (?P<zone>[\+\-]\d{4})",
    ) {
        if let Some(cap) = re.captures(s) {
            let dt = format!(
                "{} {} {} {}:{}:{} {}",
                &cap["year"],
                &cap["month"],
                &cap["day"],
                &cap["hour"],
                &cap["min"],
                &cap["sec"],
                &cap["zone"]
            );
            println!("DEBUG: {s} => {dt}");
            match DateTime::parse_from_str(&dt, "%Y %b %d %H:%M:%S %z") {
                Ok(d) => return d.timestamp(),
                Err(e) => println!("Error: {e:?}"),
            }
        }
    }

    0
}
