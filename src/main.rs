mod cli;
mod config;
mod generators;
mod log_generator;

use std::path::Path;

fn main() {
    let list_of_names: Vec<&str> = include_str!("./dataset/data/names_sampled.txt")
        .lines()
        .collect::<Vec<_>>();
    let list_of_words: Vec<&str> = include_str!("./dataset/data/words_sampled.txt")
        .lines()
        .collect::<Vec<_>>();

    let args = cli::get_cli_args();

    match args.commands {
        cli::Subcommands::Benchmark { source, eps } => {
            let (cfg, templates) = load_patterns_or_exit(&source);
            let threads = resolve_threads(source.threads);
            // `--eps N` or bare `--eps` (the config's rate) benchmarks one second's
            // worth of events at that rate; otherwise 100k events, uncapped.
            let rate = match eps {
                Some(Some(rate)) => rate,
                Some(None) => cfg.eps,
                None => 0,
            };
            let count = if rate > 0 { rate } else { 100000 };
            log_generator::benchmark(
                &templates,
                count,
                rate,
                threads,
                &list_of_names,
                &list_of_words,
            );
        }
        cli::Subcommands::Generate { source, eps } => {
            let (cfg, templates) = load_patterns_or_exit(&source);
            if let Err(e) = log_generator::generate_logs(
                eps.unwrap_or(cfg.eps),
                resolve_threads(source.threads),
                &templates,
                &list_of_names,
                &list_of_words,
            ) {
                // A closed pipe (e.g. `fakelogs generate ... | head`) is a normal way to stop.
                if e.kind() != std::io::ErrorKind::BrokenPipe {
                    eprintln!("Error writing logs: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Loads patterns from the config file and/or `--pattern` options (which replace
/// the file's patterns), printing any warnings. Exits with status 2 on errors.
fn load_patterns_or_exit(source: &cli::Source) -> (config::Config, Vec<log_generator::Template>) {
    let mut cfg = match &source.configfile {
        Some(path) => config::load(path).unwrap_or_else(|e| exit_with_error(&e)),
        None if source.patterns.is_empty() => {
            exit_with_error("no patterns given: pass a config file with -c, or a pattern with -p")
        }
        None => config::Config {
            patterns: vec![],
            eps: config::default_eps(),
        },
    };
    // Errors and warnings say where the pattern came from.
    let origin = if source.patterns.is_empty() {
        let path = source.configfile.as_deref().unwrap_or(Path::new(""));
        format!("config file {}", path.display())
    } else {
        cfg.patterns = source
            .patterns
            .iter()
            .map(|pattern| config::PatternConfig {
                pattern: pattern.clone(),
                weight: 1,
            })
            .collect();
        "--pattern".to_string()
    };

    let templates = log_generator::parse_patterns(&cfg.patterns)
        .unwrap_or_else(|e| exit_with_error(&format!("Invalid {}: {}", origin, e)));
    for (i, template) in templates.iter().enumerate() {
        for warning in &template.warnings {
            eprintln!("warning: {}: pattern {}: {}", origin, i + 1, warning);
        }
    }
    (cfg, templates)
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("error: {}", message);
    std::process::exit(2);
}

/// `--threads 0` means one thread per CPU.
fn resolve_threads(threads: usize) -> usize {
    if threads == 0 {
        std::thread::available_parallelism().map_or(1, |n| n.get())
    } else {
        threads
    }
}
