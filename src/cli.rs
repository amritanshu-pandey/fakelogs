use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fakelogs", about = "A crazy fast fake logs generator.")]
pub struct Cli {
    #[structopt(subcommand)]
    pub commands: Subcommands,
}

/// Where the patterns come from: a config file, `--pattern` options, or both.
#[derive(Debug, PartialEq, StructOpt)]
pub struct Source {
    #[structopt(short, long, help = "Path to the config file")]
    pub configfile: Option<PathBuf>,

    #[structopt(
        short = "p",
        long = "pattern",
        number_of_values = 1,
        help = "A pattern to generate; repeat for more. Replaces the config file's patterns"
    )]
    pub patterns: Vec<String>,

    #[structopt(
        short,
        long,
        default_value = "1",
        help = "Number of generator threads, 0 for one per CPU"
    )]
    pub threads: usize,
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt()]
pub enum Subcommands {
    /// Write fake logs to stdout until stopped
    Generate {
        #[structopt(flatten)]
        source: Source,

        #[structopt(
            long,
            help = "Events per second, 0 for uncapped. Overrides the config file [default: 10]"
        )]
        eps: Option<u128>,
    },
    /// Measure how fast logs can be generated, without writing them
    Benchmark {
        #[structopt(flatten)]
        source: Source,

        /// `--eps` alone uses the config file's rate; `--eps N` uses N.
        #[structopt(
            long,
            help = "Benchmark one second of events at this rate, or at the config file's rate if no value is given. Uncapped by default"
        )]
        #[allow(clippy::option_option)]
        eps: Option<Option<u128>>,
    },
}

pub fn get_cli_args() -> Cli {
    Cli::from_args()
}
