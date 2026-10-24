//! Generators append their value to `out` instead of returning a new String,
//! so rendering a log event doesn't allocate per placeholder.

use chrono::{DateTime, Datelike, Timelike, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt::Write;

// Writing to a String can't fail, so the fmt::Result from write! is ignored.

pub fn ipv4(out: &mut String, rng: &mut impl Rng) {
    let a: u8 = rng.gen_range(1..255);
    let b: u8 = rng.gen_range(1..255);
    let c: u8 = rng.gen_range(1..255);
    let d: u8 = rng.gen_range(1..255);

    let _ = write!(out, "{}.{}.{}.{}", a, b, c, d);
}

#[test]
fn test_ipv4() {
    use std::net::Ipv4Addr;

    let mut ip_addr = String::new();
    ipv4(&mut ip_addr, &mut rand::thread_rng());
    assert!(ip_addr.parse::<Ipv4Addr>().is_ok());
}

/// Full form with no `::` shortening, e.g. 2001:db8:0:0:8a2e:370:7334:1f.
pub fn ipv6(out: &mut String, rng: &mut impl Rng) {
    for i in 0..8 {
        if i > 0 {
            out.push(':');
        }
        let _ = write!(out, "{:x}", rng.gen::<u16>());
    }
}

#[test]
fn test_ipv6() {
    use std::net::Ipv6Addr;

    let mut ip_addr = String::new();
    ipv6(&mut ip_addr, &mut rand::thread_rng());
    assert!(ip_addr.parse::<Ipv6Addr>().is_ok());
}

/// `length` random lowercase hex digits.
pub fn hex(out: &mut String, rng: &mut impl Rng, length: usize) {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for _ in 0..length {
        out.push(DIGITS[rng.gen_range(0..16)] as char);
    }
}

#[test]
fn test_hex() {
    let mut out = String::new();
    hex(&mut out, &mut rand::thread_rng(), 24);
    assert_eq!(out.len(), 24);
    assert!(out
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

/// Random (version 4) UUID, e.g. 9b2e4c1a-3f5d-4e8b-a1c7-2d9f6e3b8a40.
pub fn uuid(out: &mut String, rng: &mut impl Rng) {
    let mut bits: u128 = rng.gen();
    bits = (bits & !(0xf << 76)) | (0x4 << 76); // version 4
    bits = (bits & !(0x3 << 62)) | (0x2 << 62); // RFC 4122 variant
    let _ = write!(
        out,
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        bits >> 96,
        (bits >> 80) & 0xffff,
        (bits >> 64) & 0xffff,
        (bits >> 48) & 0xffff,
        bits & 0xffff_ffff_ffff,
    );
}

#[test]
fn test_uuid() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut id = String::new();
        uuid(&mut id, &mut rng);
        let groups: Vec<usize> = id.split('-').map(str::len).collect();
        assert_eq!(groups, vec![8, 4, 4, 4, 12]);
        assert_eq!(&id[14..15], "4");
        assert!("89ab".contains(&id[19..20]));
    }
}

pub fn random_str(out: &mut String, rng: &mut impl Rng, min_length: usize, max_length: usize) {
    let length = rng.gen_range(min_length..max_length + 1);
    for _ in 0..length {
        out.push(rng.gen_range(b'A'..=b'Z') as char);
    }
}

#[test]
fn test_random_str() {
    let random = |min, max| {
        let mut out = String::new();
        random_str(&mut out, &mut rand::thread_rng(), min, max);
        out
    };

    let random_string = random(5, 10);
    assert!(random_string.len() >= 5 && random_string.len() <= 10);

    let random_string = random(5, 5);
    assert!(random_string.len() == 5);

    let random_string = random(5, 6);
    assert!(random_string.len() == 5 || random_string.len() == 6);
}

pub fn username(out: &mut String, rng: &mut impl Rng, list_of_names: &[&str]) {
    out.push_str(list_of_names.choose(rng).unwrap());
}

pub fn word(out: &mut String, rng: &mut impl Rng, list_of_words: &[&str]) {
    out.push_str(list_of_words.choose(rng).unwrap());
}

pub fn sentence(out: &mut String, rng: &mut impl Rng, list_of_words: &[&str]) {
    let number_of_words = rng.gen_range(5..15);

    for i in 0..number_of_words {
        if i > 0 {
            out.push(' ');
        }
        word(out, rng, list_of_words);
    }
    out.push('.');
}

#[test]
fn test_sentence_word_count() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut out = String::new();
        sentence(&mut out, &mut rng, &["word"]);
        assert!(out.ends_with('.'));
        let words = out.split(' ').count();
        assert!((5..15).contains(&words));
    }
}

pub fn paragraph(out: &mut String, rng: &mut impl Rng, list_of_words: &[&str]) {
    let number_of_sentences = rng.gen_range(5..15);

    for i in 0..number_of_sentences {
        if i > 0 {
            out.push(' ');
        }
        sentence(out, rng, list_of_words);
    }
}

pub fn domain(out: &mut String, rng: &mut impl Rng) {
    random_str(out, rng, 5, 10)
}

pub fn tld(out: &mut String, rng: &mut impl Rng) {
    let choice = ["com", "it", "guru", "in", "com.au", "co.in", "co.uk"];
    out.push_str(choice.choose(rng).unwrap());
}

pub fn email(out: &mut String, rng: &mut impl Rng, list_of_names: &[&str]) {
    username(out, rng, list_of_names);
    out.push('@');
    domain(out, rng);
    out.push('.');
    tld(out, rng);
}

pub fn day(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{:02}", now.day());
}

pub fn month(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{:02}", now.month());
}

pub fn month_name(out: &mut String, now: &DateTime<Utc>) {
    const NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    out.push_str(NAMES[now.month0() as usize]);
}

pub fn year(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.year());
}

pub fn hour12(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{:02}", now.hour12().1);
}

pub fn hour(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{:02}", now.hour());
}

pub fn minute(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{:02}", now.minute());
}

pub fn second(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{:02}", now.second());
}

pub fn random_int(out: &mut String, rng: &mut impl Rng, min: u64, max: u64) {
    let _ = write!(out, "{}", rng.gen_range(min..=max));
}

pub fn choice(out: &mut String, rng: &mut impl Rng, options: &[String]) {
    out.push_str(options.choose(rng).unwrap());
}

/// Seconds since the Unix epoch.
pub fn unix(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.timestamp());
}

pub fn ts_rfc3339(out: &mut String, now: &DateTime<Utc>) {
    out.push_str(&now.to_rfc3339());
}

#[test]
fn test_time_values_are_zero_padded() {
    use chrono::TimeZone;

    let now = Utc.with_ymd_and_hms(2026, 3, 4, 5, 6, 7).unwrap();
    let mut out = String::new();
    for write in [day, month, hour, hour12, minute, second, month_name] {
        write(&mut out, &now);
        out.push(' ');
    }
    assert_eq!(out, "04 03 05 05 06 07 Mar ");
}

#[test]
fn test_random_int_range() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut out = String::new();
        random_int(&mut out, &mut rng, 1024, 1026);
        assert!((1024..=1026).contains(&out.parse::<u64>().unwrap()));
    }
}
