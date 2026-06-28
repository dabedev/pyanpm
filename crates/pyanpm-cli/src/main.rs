¶mod cli;
mod output;
mod run;

use clap::Parser;

use crate::cli::Cli;

fn main() {
    std::process::exit(run::run(Cli::parse()));
}
