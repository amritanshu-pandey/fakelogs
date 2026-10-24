mod cli;
mod config;
mod generators;
mod log_generator;

fn main() {
    let list_of_names: Vec<&str> = include_str!("./dataset/data/names_sampled.txt")
        .lines()
        .collect::<Vec<_>>();
    let list_of_words: Vec<&str> = include_str!("./dataset/data/words_sampled.txt")
        .lines()
        .collect::<Vec<_>>();

    let args = cli::get_cli_args();

    match args.commands {
        cli::Subcommands::Benchmark {
            configfile,
            eps,
            threads,
        } => {
            let cfg = config::get_config_yaml(configfile);
            let templates = log_generator::parse_patterns(&cfg.patterns);
            let threads = resolve_threads(threads);
            if eps && cfg.eps > 0 {
                // Generate one second's worth of events at the configured rate.
                log_generator::benchmark(
                    &templates,
                    cfg.eps,
                    cfg.eps,
                    threads,
                    &list_of_names,
                    &list_of_words,
                );
            } else {
                log_generator::benchmark(
                    &templates,
                    100000,
                    0,
                    threads,
                    &list_of_names,
                    &list_of_words,
                );
            }
        }
        cli::Subcommands::Generate {
            configfile,
            threads,
        } => {
            let cfg = config::get_config_yaml(configfile);
            let templates = log_generator::parse_patterns(&cfg.patterns);
            if let Err(e) = log_generator::generate_logs(
                cfg.eps,
                resolve_threads(threads),
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

/// `--threads 0` means one thread per CPU.
fn resolve_threads(threads: usize) -> usize {
    if threads == 0 {
        std::thread::available_parallelism().map_or(1, |n| n.get())
    } else {
        threads
    }
}
