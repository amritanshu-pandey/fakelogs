use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fakelogs", about = "A crazy fast fake logs generator.")]
pub struct Cli {
    #[structopt(subcommand)]
    pub commands: Subcommands,
}

#[derive(Debug, PartialEq, StructOpt)]
#[structopt()]
pub enum Subcommands {
    Generate {
        /// Config file
        #[structopt(short, long, help = "Path to the config file")]
        configfile: PathBuf,
        #[structopt(
            short,
            long,
            default_value = "1",
            help = "Number of generator threads, 0 for one per CPU"
        )]
        threads: usize,
    },
    Benchmark {
        /// Config file
        #[structopt(short, long, help = "Path to the config file")]
        configfile: PathBuf,
        #[structopt(long, help = "benchmark with eps limit from config file")]
        eps: bool,
        #[structopt(
            short,
            long,
            default_value = "1",
            help = "Number of generator threads, 0 for one per CPU"
        )]
        threads: usize,
    },
}

pub fn get_cli_args() -> Cli {
    Cli::from_args()
}
