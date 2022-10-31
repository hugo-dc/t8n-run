use t8n::*;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    t8ntool: String,
    #[arg(short, long)]
    evm: Option<String>,
    #[arg(short = 'f', long)]
    hard_fork: String,
    #[arg(short = 's', long)]
    state_test: String
}

fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);
}
