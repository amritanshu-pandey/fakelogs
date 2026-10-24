use crate::generators;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::hint::black_box;
use std::io::{self, BufWriter, Write};
use std::sync::mpsc;
use std::{thread, time};

#[derive(Debug, Clone, PartialEq)]
enum Placeholder {
    Str,
    Ipv4,
    Ipv6,
    Uuid,
    Hex(usize),
    Unix,
    Email,
    Username,
    Day,
    Month,
    MonthName,
    Year,
    Hour12,
    Hour,
    Minute,
    Second,
    Int { min: u64, max: u64 },
    Choice(Vec<String>),
    TsRfc3339,
    Domain,
    Tld,
    Word,
    Sentence,
    Paragraph,
}

impl Placeholder {
    /// Parses the text between the braces, e.g. `ipv4` or `int:1-500`.
    /// Returns Ok(None) if it isn't a placeholder name, so it can be kept as literal text.
    fn parse(text: &str) -> Result<Option<Self>, String> {
        let (name, args) = match text.split_once(':') {
            Some((name, args)) => (name, Some(args)),
            None => (text, None),
        };
        let placeholder = match (name, args) {
            ("int", Some(args)) => parse_int_range(args)?,
            ("hex", Some(args)) => match args.parse() {
                Ok(length) if length > 0 => Self::Hex(length),
                _ => return Err(format!("{{{}}}: expected a length like {{hex:16}}", text)),
            },
            ("hex", None) => return Err("{hex} needs a length, e.g. {hex:16}".to_string()),
            // Empty options are allowed, e.g. {choice:|.html} for "sometimes nothing".
            ("choice", Some(args)) => Self::Choice(args.split('|').map(String::from).collect()),
            ("choice", None) => {
                return Err("{choice} needs options, e.g. {choice:GET|POST}".to_string())
            }
            (_, Some(_)) if Self::parse(name)?.is_some() => {
                return Err(format!("{{{}}}: {{{}}} doesn't take arguments", text, name))
            }
            (_, Some(_)) => return Ok(None),
            ("str", None) => Self::Str,
            ("ipv4", None) => Self::Ipv4,
            ("ipv6", None) => Self::Ipv6,
            ("uuid", None) => Self::Uuid,
            ("unix", None) => Self::Unix,
            ("email", None) => Self::Email,
            ("username", None) => Self::Username,
            ("day", None) => Self::Day,
            ("month", None) => Self::Month,
            ("month_name", None) => Self::MonthName,
            ("year", None) => Self::Year,
            ("hour12", None) => Self::Hour12,
            ("hour", None) => Self::Hour,
            ("minute", None) => Self::Minute,
            ("second", None) => Self::Second,
            ("int", None) => Self::Int { min: 1, max: 99 },
            // Misspelled name kept so existing config files keep working.
            ("ts_rfc3339" | "ts_rfc3389", None) => Self::TsRfc3339,
            ("domain", None) => Self::Domain,
            ("tld", None) => Self::Tld,
            ("word", None) => Self::Word,
            ("sentence", None) => Self::Sentence,
            ("paragraph", None) => Self::Paragraph,
            _ => return Ok(None),
        };
        Ok(Some(placeholder))
    }

    /// Appends a generated value to `out`. `now` is read at most once per event,
    /// so all time placeholders in one event agree with each other.
    fn write_to(
        &self,
        out: &mut String,
        rng: &mut impl Rng,
        now: &mut Option<DateTime<Utc>>,
        list_of_names: &[&str],
        list_of_words: &[&str],
    ) {
        let mut now = || *now.get_or_insert_with(Utc::now);
        match self {
            Self::Str => generators::random_str(out, rng, 10, 11),
            Self::Ipv4 => generators::ipv4(out, rng),
            Self::Ipv6 => generators::ipv6(out, rng),
            Self::Uuid => generators::uuid(out, rng),
            Self::Hex(length) => generators::hex(out, rng, *length),
            Self::Unix => generators::unix(out, &now()),
            Self::Email => generators::email(out, rng, list_of_names),
            Self::Username => generators::username(out, rng, list_of_names),
            Self::Day => generators::day(out, &now()),
            Self::Month => generators::month(out, &now()),
            Self::MonthName => generators::month_name(out, &now()),
            Self::Year => generators::year(out, &now()),
            Self::Hour12 => generators::hour12(out, &now()),
            Self::Hour => generators::hour(out, &now()),
            Self::Minute => generators::minute(out, &now()),
            Self::Second => generators::second(out, &now()),
            Self::Int { min, max } => generators::random_int(out, rng, *min, *max),
            Self::Choice(options) => generators::choice(out, rng, options),
            Self::TsRfc3339 => generators::ts_rfc3339(out, &now()),
            Self::Domain => generators::domain(out, rng),
            Self::Tld => generators::tld(out, rng),
            Self::Word => generators::word(out, rng, list_of_words),
            Self::Sentence => generators::sentence(out, rng, list_of_words),
            Self::Paragraph => generators::paragraph(out, rng, list_of_words),
        }
    }
}

/// Parses the `MIN-MAX` in `{int:MIN-MAX}`; both ends are included.
fn parse_int_range(args: &str) -> Result<Placeholder, String> {
    let invalid = || format!("{{int:{}}}: expected a range like {{int:1-500}}", args);
    let (min, max) = args.split_once('-').ok_or_else(invalid)?;
    let min: u64 = min.trim().parse().map_err(|_| invalid())?;
    let max: u64 = max.trim().parse().map_err(|_| invalid())?;
    if min > max {
        return Err(format!("{{int:{}}}: {} is bigger than {}", args, min, max));
    }
    Ok(Placeholder::Int { min, max })
}

#[derive(Debug, PartialEq)]
enum Token {
    Literal(String),
    Placeholder(Placeholder),
}

/// A log pattern parsed once into literal text and placeholders, so each
/// event only has to fill in the placeholders.
#[derive(Debug)]
pub struct Template {
    tokens: Vec<Token>,
}

impl Template {
    /// Unknown `{...}` sequences (like the braces in a JSON pattern) are kept as literal text.
    /// Known placeholders with bad arguments, like `{int:9-1}`, are an error.
    pub fn parse(pattern: &str) -> Result<Self, String> {
        let mut tokens = vec![];
        let mut literal = String::new();
        let mut rest = pattern;

        while let Some(open) = rest.find('{') {
            literal.push_str(&rest[..open]);
            let after_open = &rest[open + 1..];
            let placeholder = match after_open.find('}') {
                Some(close) => Placeholder::parse(&after_open[..close])?.map(|p| (p, close)),
                None => None,
            };
            match placeholder {
                Some((placeholder, close)) => {
                    if !literal.is_empty() {
                        tokens.push(Token::Literal(std::mem::take(&mut literal)));
                    }
                    tokens.push(Token::Placeholder(placeholder));
                    rest = &after_open[close + 1..];
                }
                None => {
                    literal.push('{');
                    rest = after_open;
                }
            }
        }
        literal.push_str(rest);
        if !literal.is_empty() {
            tokens.push(Token::Literal(literal));
        }

        Ok(Self { tokens })
    }

    /// Appends one rendered log event (without a newline) to `out`.
    pub fn render_into(&self, out: &mut String, list_of_names: &[&str], list_of_words: &[&str]) {
        let mut rng = rand::thread_rng();
        let mut now = None;
        for token in &self.tokens {
            match token {
                Token::Literal(text) => out.push_str(text),
                Token::Placeholder(placeholder) => {
                    placeholder.write_to(out, &mut rng, &mut now, list_of_names, list_of_words)
                }
            }
        }
    }

    #[cfg(test)]
    pub fn render(&self, list_of_names: &[&str], list_of_words: &[&str]) -> String {
        let mut out = String::new();
        self.render_into(&mut out, list_of_names, list_of_words);
        out
    }
}

pub fn parse_patterns(patterns: &[String]) -> Result<Vec<Template>, String> {
    patterns.iter().map(|p| Template::parse(p)).collect()
}

/// Paces events to a target rate by sleeping until each event's deadline,
/// so time spent generating doesn't eat into the rate.
struct Pacer {
    interval: time::Duration,
    next_deadline: time::Instant,
}

impl Pacer {
    fn new(eps: u128) -> Option<Self> {
        let interval = time::Duration::from_nanos(
            1_000_000_000_u128
                .checked_div(eps)?
                .try_into()
                .expect("Unable to coerce u128 into u64"),
        );
        Some(Self {
            interval,
            next_deadline: time::Instant::now(),
        })
    }

    /// How long to sleep before the next event, or None if we are behind schedule.
    fn delay(&mut self) -> Option<time::Duration> {
        self.next_deadline += self.interval;
        let now = time::Instant::now();
        if now < self.next_deadline {
            Some(self.next_deadline - now)
        } else {
            // After a long stall (e.g. a blocked stdout), don't burst to catch up.
            if now - self.next_deadline > time::Duration::from_secs(1) {
                self.next_deadline = now;
            }
            None
        }
    }
}

/// Splits `total` into `parts` shares that differ by at most one.
fn split_evenly(total: u128, parts: usize) -> Vec<u128> {
    let parts_u128 = parts as u128;
    (0..parts_u128)
        .map(|i| total / parts_u128 + u128::from(i < total % parts_u128))
        .collect()
}

/// Per-thread event rates. With a cap, there are never more threads than
/// events per second, so every thread gets a nonzero (capped) share.
fn thread_rates(eps: u128, threads: usize) -> Vec<u128> {
    if eps == 0 {
        vec![0; threads]
    } else {
        split_evenly(eps, threads.min(eps.try_into().unwrap_or(usize::MAX)))
    }
}

/// Batches are handed from generator threads to the writer at about this size.
const BATCH_SIZE: usize = 64 * 1024;

/// Generates newline-terminated events into batches and sends them to the
/// writer until the writer hangs up.
fn generate_batches(
    batches: mpsc::SyncSender<String>,
    eps: u128,
    templates: &[Template],
    list_of_names: &[&str],
    list_of_words: &[&str],
) {
    let mut pacer = Pacer::new(eps);
    let mut batch = String::with_capacity(BATCH_SIZE);

    loop {
        for template in templates {
            template.render_into(&mut batch, list_of_names, list_of_words);
            batch.push('\n');
            let delay = pacer.as_mut().and_then(Pacer::delay);
            // Send before idling so rate-limited output arrives steadily
            // instead of in batch-sized bursts.
            if batch.len() >= BATCH_SIZE || delay.is_some() {
                let full = std::mem::replace(&mut batch, String::with_capacity(BATCH_SIZE));
                if batches.send(full).is_err() {
                    return;
                }
            }
            if let Some(delay) = delay {
                thread::sleep(delay);
            }
        }
    }
}

/// Writes batches to stdout in the order they arrive, flushing whenever no
/// batch is waiting.
fn write_batches(batches: mpsc::Receiver<String>) -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout().lock());
    loop {
        let batch = match batches.try_recv() {
            Ok(batch) => batch,
            Err(mpsc::TryRecvError::Empty) => {
                out.flush()?;
                match batches.recv() {
                    Ok(batch) => batch,
                    Err(_) => break,
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => break,
        };
        out.write_all(batch.as_bytes())?;
    }
    out.flush()
}

pub fn generate_logs(
    eps: u128,
    threads: usize,
    templates: &[Template],
    list_of_names: &[&str],
    list_of_words: &[&str],
) -> io::Result<()> {
    let rates = thread_rates(eps, threads);
    let (sender, receiver) = mpsc::sync_channel(rates.len() * 4);

    thread::scope(|scope| {
        for rate in rates {
            let sender = sender.clone();
            scope.spawn(move || {
                generate_batches(sender, rate, templates, list_of_names, list_of_words)
            });
        }
        drop(sender);
        // Returning drops the receiver, which stops the generator threads.
        write_batches(receiver)
    })
}

/// Renders `count` events at up to `eps` events per second (0 = uncapped)
/// and returns how many were generated.
fn benchmark_thread(
    count: u128,
    eps: u128,
    templates: &[Template],
    list_of_names: &[&str],
    list_of_words: &[&str],
) -> u128 {
    let mut counter: u128 = 0;
    let mut pacer = Pacer::new(eps);
    let mut line = String::new();
    while counter < count {
        for template in templates {
            line.clear();
            template.render_into(&mut line, list_of_names, list_of_words);
            black_box(&line);
            counter += 1;
            if let Some(delay) = pacer.as_mut().and_then(Pacer::delay) {
                thread::sleep(delay);
            }
        }
    }
    counter
}

pub fn benchmark(
    templates: &[Template],
    count: u128,
    eps: u128,
    threads: usize,
    list_of_names: &[&str],
    list_of_words: &[&str],
) {
    let rates = thread_rates(eps, threads.min(count.try_into().unwrap_or(usize::MAX)));
    let counts = split_evenly(count, rates.len());
    eprintln!("Benchmarking with {} thread(s) ...", rates.len());

    let start_time = time::Instant::now();
    let counter: u128 = thread::scope(|scope| {
        let workers: Vec<_> = counts
            .into_iter()
            .zip(rates)
            .map(|(count, rate)| {
                scope.spawn(move || {
                    benchmark_thread(count, rate, templates, list_of_names, list_of_words)
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).sum()
    });
    let duration = start_time.elapsed();
    eprintln!(
        "{} log events generated in {} seconds",
        counter,
        duration.as_secs_f64()
    );
    eprintln!(
        "EPS: {}\n",
        (counter as f64 / duration.as_secs_f64()).round()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: &[&str] = &["Alice"];
    const WORDS: &[&str] = &["word"];

    #[test]
    fn test_parse_keeps_unknown_braces_as_literals() {
        let template = Template::parse(r#"{"user":"{username}", "x":"{nope}"}"#).unwrap();
        assert_eq!(
            template.tokens,
            vec![
                Token::Literal(r#"{"user":""#.to_string()),
                Token::Placeholder(Placeholder::Username),
                Token::Literal(r#"", "x":"{nope}"}"#.to_string()),
            ]
        );
    }

    #[test]
    fn test_render_replaces_every_placeholder() {
        let template = Template::parse("{username} {word} {username} {ts_rfc3389}").unwrap();
        let out = template.render(NAMES, WORDS);
        assert!(out.starts_with("Alice word Alice "));
        assert!(!out.contains('{'));
    }

    #[test]
    fn test_render_generates_each_ipv4_separately() {
        let template = Template::parse("{ipv4} {ipv4} {ipv4} {ipv4}").unwrap();
        let distinct = (0..10).map(|_| template.render(NAMES, WORDS)).any(|out| {
            let ips: Vec<&str> = out.split(' ').collect();
            ips.iter().any(|ip| *ip != ips[0])
        });
        assert!(distinct);
    }

    #[test]
    fn test_parse_placeholders_with_arguments() {
        let template = Template::parse("{int:1024-65535} {choice:GET|POST} {int}").unwrap();
        assert_eq!(
            template.tokens,
            vec![
                Token::Placeholder(Placeholder::Int {
                    min: 1024,
                    max: 65535
                }),
                Token::Literal(" ".to_string()),
                Token::Placeholder(Placeholder::Choice(vec![
                    "GET".to_string(),
                    "POST".to_string()
                ])),
                Token::Literal(" ".to_string()),
                Token::Placeholder(Placeholder::Int { min: 1, max: 99 }),
            ]
        );
        let out = template.render(NAMES, WORDS);
        assert!(out.contains(" GET ") || out.contains(" POST "));
    }

    #[test]
    fn test_parse_rejects_bad_arguments() {
        for pattern in [
            "{int:9-1}",
            "{int:x-5}",
            "{int:5}",
            "{choice}",
            "{hour:2}",
            "{hex}",
            "{hex:0}",
        ] {
            assert!(Template::parse(pattern).is_err(), "{}", pattern);
        }
        // Unknown names with a colon, like JSON keys, are still literal text.
        assert!(Template::parse(r#"{"level":"info"}"#).is_ok());
    }

    #[test]
    fn test_split_evenly() {
        assert_eq!(split_evenly(10, 3), vec![4, 3, 3]);
        assert_eq!(split_evenly(0, 2), vec![0, 0]);
    }

    #[test]
    fn test_thread_rates_never_uncaps_a_thread() {
        assert_eq!(thread_rates(0, 3), vec![0, 0, 0]);
        assert_eq!(thread_rates(2, 4), vec![1, 1]);
        assert_eq!(thread_rates(1500, 4), vec![375, 375, 375, 375]);
    }

    #[test]
    fn test_unterminated_brace() {
        let template = Template::parse("a {b").unwrap();
        assert_eq!(template.render(NAMES, WORDS), "a {b");
    }

    #[test]
    fn test_example_configs_are_valid() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("example");
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "yaml") {
                let cfg = crate::config::get_config_yaml(path.clone());
                let templates = parse_patterns(&cfg.patterns)
                    .unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
                for template in &templates {
                    let out = template.render(NAMES, WORDS);
                    assert!(!out.is_empty(), "{}", path.display());
                }
                checked += 1;
            }
        }
        assert!(checked > 0);
    }
}
