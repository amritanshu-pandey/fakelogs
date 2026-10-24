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
    let _ = write!(out, "{}", now.day());
}

pub fn month(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.month());
}

pub fn year(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.year());
}

pub fn hour12(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.hour12().1);
}

pub fn hour(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.hour());
}

pub fn minute(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.minute());
}

pub fn second(out: &mut String, now: &DateTime<Utc>) {
    let _ = write!(out, "{}", now.second());
}

pub fn random_int(out: &mut String, rng: &mut impl Rng) {
    let _ = write!(out, "{}", rng.gen_range(1..100));
}

pub fn ts_rfc3339(out: &mut String, now: &DateTime<Utc>) {
    out.push_str(&now.to_rfc3339());
}
