use crate::generators;
use std::io::{self, BufWriter, Write};
use std::{thread, time};

#[derive(Debug)]
pub struct Log {
    pub pattern: String,
}

/// Paces events to a target rate by sleeping until each event's deadline,
/// so time spent generating doesn't eat into the rate.
struct Pacer {
    interval: time::Duration,
    next_deadline: time::Instant,
}

impl Pacer {
    fn new(eps: u128) -> Option<Self> {
        if eps == 0 {
            return None;
        }
        let interval = time::Duration::from_nanos(
            (1_000_000_000 / eps)
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
    patterns: &Vec<String>,
    list_of_names: &Vec<&str>,
    list_of_words: &Vec<&str>,
) -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout().lock());
    let mut pacer = Pacer::new(eps);

    loop {
        for pattern in patterns {
            writeln!(
                out,
                "{}",
                Log::init(pattern.to_string()).generate(list_of_names, list_of_words)
            )?;
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
    patterns: &Vec<String>,
    count: u128,
    list_of_names: &Vec<&str>,
    list_of_words: &Vec<&str>,
) {
    let mut counter: u128 = 0;
    let start_time = time::Instant::now();
    eprintln!("Benchmarking ...");
    loop {
        for pattern in patterns {
            Log::init(pattern.to_string()).generate(list_of_names, list_of_words);
            counter += 1;
        }
        if counter >= count {
            break;
        }
    }
    let end_time = time::Instant::now();
    let duration = end_time - start_time;
    eprintln!(
        "{} log events generated in {} seconds",
        count,
        duration.as_secs_f64()
    );
    eprintln!("EPS: {}\n", (count as f64 / duration.as_secs_f64()).round());
}

pub fn benchmark_with_eps(
    patterns: &Vec<String>,
    eps: u128,
    list_of_names: &Vec<&str>,
    list_of_words: &Vec<&str>,
) {
    let count: u128 = eps;
    let mut counter: u128 = 0;
    let start_time = time::Instant::now();
    eprintln!("Benchmarking ...");
    let mut pacer = Pacer::new(eps);
    loop {
        for pattern in patterns {
            Log::init(pattern.to_string()).generate(list_of_names, list_of_words);
            counter += 1;
            if let Some(delay) = pacer.as_mut().and_then(Pacer::delay) {
                thread::sleep(delay);
            }
        }
        if counter >= count {
            break;
        }
    }
    let end_time = time::Instant::now();
    let duration = end_time - start_time;
    eprintln!(
        "{} log events generated in {} seconds",
        count,
        duration.as_secs_f64()
    );
    eprintln!("EPS: {}\n", (count as f64 / duration.as_secs_f64()).round());
}

fn replace_all(
    pattern: &str,
    identifier: &str,
    list_of_names: &Vec<&str>,
    list_of_words: &Vec<&str>,
) -> String {
    let out: String = match identifier {
        "{ipv4}" => pattern.replacen("{ipv4}", &generators::ipv4(), 1),
        "{str}" => pattern.replacen("{str}", &generators::random_str(10, 11), 1),
        "{email}" => pattern.replacen("{email}", &generators::email(&list_of_names), 1),
        "{domain}" => pattern.replacen("{domain}", &generators::domain(), 1),
        "{tld}" => pattern.replacen("{tld}", &generators::tld(), 1),
        "{username}" => pattern.replacen("{username}", &generators::username(list_of_names), 1),
        "{day}" => pattern.replacen("{day}", &generators::day().to_string(), 1),
        "{month}" => pattern.replacen("{month}", &generators::month().to_string(), 1),
        "{year}" => pattern.replacen("{year}", &generators::year().to_string(), 1),
        "{hour12}" => pattern.replacen("{hour12}", &generators::hour12().1.to_string(), 1),
        "{hour}" => pattern.replacen("{hour}", &generators::hour().to_string(), 1),
        "{minute}" => pattern.replacen("{minute}", &generators::minute().to_string(), 1),
        "{second}" => pattern.replacen("{second}", &generators::second().to_string(), 1),
        "{int}" => pattern.replacen("{int}", &generators::random_int().to_string(), 1),
        "{ts_rfc3339}" => pattern.replacen("{ts_rfc3339}", &generators::ts_rfc3339(), 1),
        // Misspelled name kept so existing config files keep working.
        "{ts_rfc3389}" => pattern.replacen("{ts_rfc3389}", &generators::ts_rfc3339(), 1),
        "{word}" => pattern.replacen("{word}", &generators::word(&list_of_words).to_string(), 1),
        "{sentence}" => pattern.replacen(
            "{sentence}",
            &generators::sentence(&list_of_words).to_string(),
            1,
        ),
        "{paragraph}" => pattern.replacen(
            "{paragraph}",
            &generators::paragraph(&list_of_words).to_string(),
            1,
        ),
        _ => pattern.to_string(),
    };

    if out.contains(identifier) {
        replace_all(&out, identifier, &list_of_names, &list_of_words)
    } else {
        out.to_owned()
    }
}

impl Log {
    pub fn init(pattern: String) -> Self {
        Self { pattern: pattern }
    }

    pub fn generate(&self, list_of_names: &Vec<&str>, list_of_words: &Vec<&str>) -> String {
        let mut output = self.pattern.to_string();
        let identifiers: Vec<&str> = vec![
            "{str}",
            "{ipv4}",
            "{email}",
            "{username}",
            "{day}",
            "{month}",
            "{year}",
            "{hour12}",
            "{hour}",
            "{minute}",
            "{second}",
            "{int}",
            "{ts_rfc3339}",
            "{ts_rfc3389}",
            "{domain}",
            "{tld}",
            "{word}",
            "{sentence}",
            "{paragraph}",
        ];
        for identifier in identifiers {
            output = replace_all(&output, identifier, &list_of_names, &list_of_words);
        }
        output
    }
}
