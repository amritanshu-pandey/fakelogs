use chrono::{self, Datelike, Timelike};
use rand::seq::SliceRandom;
use rand::Rng;
use std::net::{IpAddr, Ipv4Addr};

pub fn ipv4() -> String {
    let mut rnd = rand::thread_rng();
    let a: i32 = rnd.gen_range(1..255);
    let b: i32 = rnd.gen_range(1..255);
    let c: i32 = rnd.gen_range(1..255);
    let d: i32 = rnd.gen_range(1..255);

    format!("{}.{}.{}.{}", a, b, c, d)
}

#[test]
fn test_ipv4() {
    let ip_addr = ipv4();
    let random_ipv4: Vec<&str> = ip_addr.split(".").collect();
    assert_eq!(
        IpAddr::V4(Ipv4Addr::new(
            random_ipv4[0].parse::<u8>().unwrap(),
            random_ipv4[1].parse::<u8>().unwrap(),
            random_ipv4[2].parse::<u8>().unwrap(),
            random_ipv4[3].parse::<u8>().unwrap()
        ))
        .is_ipv4(),
        true
    );
}

pub fn random_str(min_length: usize, max_length: usize) -> String {
    let mut rnd = rand::thread_rng();
    let length = rnd.gen_range(min_length..max_length + 1);
    random_string::generate(length, "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string())
}

#[test]
fn test_random_str() {
    let random_string = random_str(5, 10);
    assert!(random_string.len() >= 5 && random_string.len() <= 10);

    let random_string = random_str(5, 5);
    assert!(random_string.len() == 5);

    let random_string = random_str(5, 6);
    assert!(random_string.len() == 5 || random_string.len() == 6);
}

pub fn username(list_of_names: &Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    list_of_names.choose(&mut rng).unwrap().to_string()
}

pub fn word(list_of_words: &Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    list_of_words.choose(&mut rng).unwrap().to_string()
}

pub fn sentence(list_of_words: &Vec<&str>) -> String {
    let connector = " ";
    let punctuation = ".";

    let mut rnd = rand::thread_rng();
    let number_of_words = rnd.gen_range(5..15);

    let mut words: Vec<String> = vec![];
    for _ in 1..number_of_words {
        words.push(word(&list_of_words));
    }
    format!("{}{}", words.join(connector), punctuation)
}

pub fn paragraph(list_of_words: &Vec<&str>) -> String {
    let connector: &str = " ";

    let mut rnd = rand::thread_rng();
    let number_of_sentences = rnd.gen_range(5..15);

    let mut sentences: Vec<String> = vec![];
    for _ in 1..number_of_sentences {
        sentences.push(sentence(&list_of_words));
    }
    sentences.join(connector)
}

pub fn domain() -> String {
    random_str(5, 10)
}

pub fn tld() -> String {
    let choice = vec!["com", "it", "guru", "in", "com.au", "co.in", "co.uk"];
    let mut rng = rand::thread_rng();
    choice.choose(&mut rng).unwrap().to_string()
}

pub fn email(list_of_names: &Vec<&str>) -> String {
    format!("{}@{}.{}", username(list_of_names), domain(), tld())
}

pub fn day() -> u32 {
    chrono::Utc::now().day()
}

pub fn month() -> u32 {
    chrono::Utc::now().month()
}

pub fn year() -> i32 {
    chrono::Utc::now().year()
}

pub fn hour12() -> (bool, u32) {
    chrono::Utc::now().hour12()
}

pub fn hour() -> u32 {
    chrono::Utc::now().hour()
}

pub fn minute() -> u32 {
    chrono::Utc::now().minute()
}

pub fn second() -> u32 {
    chrono::Utc::now().second()
}

pub fn random_int() -> String {
    let mut rnd = rand::thread_rng();
    rnd.gen_range(1..100).to_string()
}

pub fn ts_rfc3389() -> String {
    chrono::Utc::now().to_rfc3339()
}
