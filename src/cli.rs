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
    },
    Benchmark {
        /// Config file
        #[structopt(short, long, help = "Path to the config file")]
        configfile: PathBuf,
        #[structopt(long, help = "benchmark with eps limit from config file")]
        eps: bool,
    },
}

pub fn get_cli_args() -> Cli {
    let cli_args = Cli::from_args();
    cli_args
}
