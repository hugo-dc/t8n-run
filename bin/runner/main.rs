use t8n::context::Context;

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
    let context_result = Context::from_state_test(args.state_test.as_str());

    if context_result.is_ok() {
        let mut context = context_result.unwrap();
        if args.evm.is_some() {
            context.config.evm = args.evm.unwrap().to_string();
        }
        context.run();
    } else {
        println!("Error getting information from state test file");
    }
}
