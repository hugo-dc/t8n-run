use t8n::context::{Alloc, Context, TransactionT8n};

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    t8ntool: String,
    #[arg(short, long)]
    data: Option<String>,
    #[arg(short, long)]
    code: Option<String>,
    #[arg(short, long)]
    gas: Option<String>,
    #[arg(short, long)]
    evm: Option<String>,
    #[arg(short = 'f', long)]
    hard_fork: String,
    #[arg(short = 's', long)]
    state_test: Option<String>
}

fn main() {
    let args = Args::parse();

    let context_result: Result<Context, &'static str>;

    if args.state_test.is_some() {
        context_result = Context::from_state_test(args.state_test.unwrap().as_str());
        if context_result.is_err() {
            println!("Error getting information from state test file");
            return;
        }
    } else {
        context_result = Ok(Context::default());
    }

    let mut ctx = context_result.unwrap();

    // Set Hard Fork
    ctx.config.hard_fork = args.hard_fork.clone();

    // Set EVM if provided
    if args.evm.is_some() {
        ctx.config.evm = args.evm.unwrap().to_string();
    }

    // Set code if provided
    if args.code.is_some() || args.data.is_some() {
        let _ = ctx.add_default_address(); 
        // Create sender account
        let snd_address = String::from("0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b");

        // Create default transaction
        ctx.txs.push(TransactionT8n::default());

        // Set sender
        let pk = ctx.get_secret_key(snd_address);
        ctx.txs[0].set_private_key(pk.unwrap().as_str());

        if args.code.is_some() {
            // Create receiver account
            let rec_address = String::from("0x0000000000000000000000000000000000000100");
            let _ = ctx.add_address(&rec_address);
            let mut account: Alloc = ctx.alloc.get(&rec_address).unwrap().clone();
            let _ = account.set_code(args.code.unwrap()); 
            let _ = ctx.alloc.remove(&rec_address);
            ctx.alloc.insert(rec_address.clone(), account.to_owned());
            // Set receiver
            ctx.txs[0].set_receiver(&rec_address);
        }

        if args.data.is_some() {
            // Set input data
            ctx.txs[0].set_input(args.data.unwrap().as_str());
        }
    }

    if args.gas.is_some() {
        ctx.txs[0].set_gas(&args.gas.unwrap());
    }


    ctx.run();
}
