mod cli;
mod config;
mod generators;
mod log_generator;

fn main() {
    let list_of_names: Vec<&str> = include_str!("./dataset/data/names_sampled.txt")
        .split("\n")
        .collect::<Vec<_>>();
    let list_of_words: Vec<&str> = include_str!("./dataset/data/words_sampled.txt")
        .split("\n")
        .collect::<Vec<_>>();

    let args = cli::get_cli_args();

    match args.commands {
        cli::Subcommands::Benchmark { configfile, eps } => {
            let cfg = config::get_config_yaml(configfile);
            if eps {
                log_generator::benchmark_with_eps(
                    &cfg.patterns,
                    cfg.eps,
                    &list_of_names,
                    &list_of_words,
                );
            } else {
                log_generator::benchmark(&cfg.patterns, 100000, &list_of_names, &list_of_words);
            }
        }
        cli::Subcommands::Generate { configfile } => {
            let cfg = config::get_config_yaml(configfile);
            log_generator::generate_logs(cfg.eps, &cfg.patterns, &list_of_names, &list_of_words);
        }
    }
}
