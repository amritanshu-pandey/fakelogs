use crate::generators;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::hint::black_box;
use std::io::{self, BufWriter, Write};
use std::{thread, time};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Placeholder {
    Str,
    Ipv4,
    Email,
    Username,
    Day,
    Month,
    Year,
    Hour12,
    Hour,
    Minute,
    Second,
    Int,
    TsRfc3339,
    Domain,
    Tld,
    Word,
    Sentence,
    Paragraph,
}

impl Placeholder {
    /// Looks up a placeholder by its name without the surrounding braces.
    fn from_name(name: &str) -> Option<Self> {
        let placeholder = match name {
            "str" => Self::Str,
            "ipv4" => Self::Ipv4,
            "email" => Self::Email,
            "username" => Self::Username,
            "day" => Self::Day,
            "month" => Self::Month,
            "year" => Self::Year,
            "hour12" => Self::Hour12,
            "hour" => Self::Hour,
            "minute" => Self::Minute,
            "second" => Self::Second,
            "int" => Self::Int,
            // Misspelled name kept so existing config files keep working.
            "ts_rfc3339" | "ts_rfc3389" => Self::TsRfc3339,
            "domain" => Self::Domain,
            "tld" => Self::Tld,
            "word" => Self::Word,
            "sentence" => Self::Sentence,
            "paragraph" => Self::Paragraph,
            _ => return None,
        };
        Some(placeholder)
    }

    /// Appends a generated value to `out`. `now` is read at most once per event,
    /// so all time placeholders in one event agree with each other.
    fn write_to(
        self,
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
            Self::Email => generators::email(out, rng, list_of_names),
            Self::Username => generators::username(out, rng, list_of_names),
            Self::Day => generators::day(out, &now()),
            Self::Month => generators::month(out, &now()),
            Self::Year => generators::year(out, &now()),
            Self::Hour12 => generators::hour12(out, &now()),
            Self::Hour => generators::hour(out, &now()),
            Self::Minute => generators::minute(out, &now()),
            Self::Second => generators::second(out, &now()),
            Self::Int => generators::random_int(out, rng),
            Self::TsRfc3339 => generators::ts_rfc3339(out, &now()),
            Self::Domain => generators::domain(out, rng),
            Self::Tld => generators::tld(out, rng),
            Self::Word => generators::word(out, rng, list_of_words),
            Self::Sentence => generators::sentence(out, rng, list_of_words),
            Self::Paragraph => generators::paragraph(out, rng, list_of_words),
        }
    }
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
    pub fn parse(pattern: &str) -> Self {
        let mut tokens = vec![];
        let mut literal = String::new();
        let mut rest = pattern;

        while let Some(open) = rest.find('{') {
            literal.push_str(&rest[..open]);
            let after_open = &rest[open + 1..];
            let placeholder = after_open
                .find('}')
                .and_then(|close| Some((Placeholder::from_name(&after_open[..close])?, close)));
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

        Self { tokens }
    }

    /// Renders one log event into `out`, replacing its contents.
    pub fn render_into(&self, out: &mut String, list_of_names: &[&str], list_of_words: &[&str]) {
        out.clear();
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

pub fn parse_patterns(patterns: &[String]) -> Vec<Template> {
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

pub fn generate_logs(
    eps: u128,
    templates: &[Template],
    list_of_names: &[&str],
    list_of_words: &[&str],
) -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout().lock());
    let mut pacer = Pacer::new(eps);
    let mut line = String::new();

    loop {
        for template in templates {
            template.render_into(&mut line, list_of_names, list_of_words);
            writeln!(out, "{}", line)?;
            if let Some(delay) = pacer.as_mut().and_then(Pacer::delay) {
                // Flush before idling so rate-limited output arrives steadily
                // instead of in buffer-sized bursts.
                out.flush()?;
                thread::sleep(delay);
            }
        }
    }
}

pub fn benchmark(
    templates: &[Template],
    count: u128,
    eps: u128,
    list_of_names: &[&str],
    list_of_words: &[&str],
) {
    let mut counter: u128 = 0;
    let mut pacer = Pacer::new(eps);
    let mut line = String::new();
    let start_time = time::Instant::now();
    eprintln!("Benchmarking ...");
    while counter < count {
        for template in templates {
            template.render_into(&mut line, list_of_names, list_of_words);
            black_box(&line);
            counter += 1;
            if let Some(delay) = pacer.as_mut().and_then(Pacer::delay) {
                thread::sleep(delay);
            }
        }
    }
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
        let template = Template::parse(r#"{"user":"{username}", "x":"{nope}"}"#);
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
        let template = Template::parse("{username} {word} {username} {ts_rfc3389}");
        let out = template.render(NAMES, WORDS);
        assert!(out.starts_with("Alice word Alice "));
        assert!(!out.contains('{'));
    }

    #[test]
    fn test_render_generates_each_ipv4_separately() {
        let template = Template::parse("{ipv4} {ipv4} {ipv4} {ipv4}");
        let distinct = (0..10).map(|_| template.render(NAMES, WORDS)).any(|out| {
            let ips: Vec<&str> = out.split(' ').collect();
            ips.iter().any(|ip| *ip != ips[0])
        });
        assert!(distinct);
    }

    #[test]
    fn test_unterminated_brace() {
        let template = Template::parse("a {b");
        assert_eq!(template.render(NAMES, WORDS), "a {b");
    }
}
