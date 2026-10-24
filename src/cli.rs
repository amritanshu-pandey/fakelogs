use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "fakelogs",
    about = "A crazy fast fake logs generator.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Subcommands,
}

/// Where the patterns come from, and how many events to make.
#[derive(Debug, PartialEq, Args)]
pub struct Source {
    #[arg(short, long, help = "Path to the config file")]
    pub configfile: Option<PathBuf>,

    #[arg(
        short = 'p',
        long = "pattern",
        help = "A pattern to generate; repeat for more. Replaces the config file's patterns"
    )]
    pub patterns: Vec<String>,

    #[arg(
        short,
        long,
        default_value_t = 1,
        help = "Number of generator threads, 0 for one per CPU"
    )]
    pub threads: usize,

    #[arg(
        short = 'n',
        long,
        help = "Number of events to generate, then exit. generate runs until stopped by default, benchmark does 100000"
    )]
    pub count: Option<u128>,

    #[arg(
        long,
        help = "Seed for the random values, for repeatable output with the same --threads. Times still come from the clock"
    )]
    pub seed: Option<u64>,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum Subcommands {
    /// Write fake logs to stdout until stopped
    Generate {
        #[command(flatten)]
        source: Source,

        #[arg(
            long,
            help = "Events per second, 0 for uncapped. Overrides the config file [default: 10]"
        )]
        eps: Option<u128>,
    },
    /// Measure how fast logs can be generated, without writing them
    Benchmark {
        #[command(flatten)]
        source: Source,

        /// `--eps` alone uses the config file's rate; `--eps N` uses N.
        #[arg(
            long,
            help = "Benchmark one second of events at this rate, or at the config file's rate if no value is given. Uncapped by default"
        )]
        #[allow(clippy::option_option)]
        eps: Option<Option<u128>>,
    },
}

pub fn get_cli_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn parse(args: &[&str]) -> Subcommands {
        Cli::try_parse_from(std::iter::once("fakelogs").chain(args.iter().copied()))
            .unwrap()
            .commands
    }

    #[test]
    fn test_cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_generate_options() {
        let Subcommands::Generate { source, eps } = parse(&[
            "generate", "-c", "a.yaml", "-p", "x {ipv4}", "-p", "y", "--eps", "50", "-t", "4",
            "-n", "1000", "--seed", "7",
        ]) else {
            panic!("expected generate");
        };
        assert_eq!(source.configfile, Some(PathBuf::from("a.yaml")));
        assert_eq!(source.patterns, vec!["x {ipv4}", "y"]);
        assert_eq!(
            (source.threads, source.count, source.seed, eps),
            (4, Some(1000), Some(7), Some(50))
        );
    }

    #[test]
    fn test_generate_defaults() {
        let Subcommands::Generate { source, eps } = parse(&["generate", "-p", "x"]) else {
            panic!("expected generate");
        };
        assert_eq!(
            (
                source.configfile,
                source.threads,
                source.count,
                source.seed,
                eps
            ),
            (None, 1, None, None, None)
        );
    }

    #[test]
    fn test_benchmark_eps_forms() {
        let eps = |args: &[&str]| match parse(args) {
            Subcommands::Benchmark { eps, .. } => eps,
            _ => panic!("expected benchmark"),
        };
        assert_eq!(eps(&["benchmark", "-c", "a.yaml"]), None);
        assert_eq!(eps(&["benchmark", "-c", "a.yaml", "--eps"]), Some(None));
        assert_eq!(eps(&["benchmark", "--eps", "-c", "a.yaml"]), Some(None));
        assert_eq!(
            eps(&["benchmark", "-c", "a.yaml", "--eps", "500"]),
            Some(Some(500))
        );
    }

    #[test]
    fn test_rejects_bad_values() {
        for args in [
            &["generate", "-p", "x", "--eps", "fast"][..],
            &["generate", "-p", "x", "-n", "-5"][..],
            &["generate", "-p", "x", "--threads", "many"][..],
        ] {
            assert!(
                Cli::try_parse_from(std::iter::once("fakelogs").chain(args.iter().copied()))
                    .is_err(),
                "{:?}",
                args
            );
        }
    }
}
