use std::io;
use std::process::Command;
use std::io::Write;
use std::fs;

use crate::context::{Alloc, Context, TransactionT8n};

struct ReplCommand {
    command_name: String,
    command_params: Vec<String>,
}

impl ReplCommand {
    pub fn from_string(st : String) -> Option<ReplCommand> {
        let valid_commands = vec!["exit", "help", "extract", "dir", "alloc", "alloc.add", "alloc.add.default", "addcode", "env", "env.set.difficulty", "env.set.currentrandom", "txs", "tx.new", "tx.set.sender", "tx.set.receiver", "tx.set.input", "tx.set.value", "hf", "run", "save", "load", "t8n", "evm"];

        let input_command = st.clone();
        let input_command = input_command.trim();
        let words: Vec<&str> = input_command.split(' ').collect();

        let cmd : String;
        let mut params: Vec<String> = Vec::new();
        if words.len() == 0 {
            cmd = st.trim().to_string(); 
        } else {
            cmd = words[0].trim().to_string();
            if words.len() > 1 {
                for w in words.iter().skip(1) {
                    params.push(w.to_string());
                }
            }
        }

        if valid_commands.iter().find(|&&s| s == cmd.as_str()).is_some() {
           return Some(ReplCommand { command_name : cmd, command_params: params });
        }
        return None;
    }

    fn check_params(&self, total: usize, param_names: &str) -> Result<(), ()>{
        if self.command_params.len() != total {
            println!("Error: Expected {} parameters ({}).", total, param_names);
            return Err(())
        }
        return Ok(())
    }

    fn cmd_help(&self) {
        println!("\thelp\t\t\tShows this help");
        println!("\tdir <path>\t\tSets <path> as the current working directory");
        println!("\textract <test>\t\tExtract context information from Ethereum State Test");
        println!("\thf <hf_name>\t\tSet HardFork");
        println!("\tt8n <t8n path>\t\tSet t8n tool path");
        println!("\tevm <evm_path>\t\tSet custom EVMC vm");
        println!("\talloc\t\t\tShow current allocation data");
        println!("\talloc.add [address]\tCreates new account");
        println!("\talloc.add.default\tCreates default account");
        println!("\taddcode <address> <bytecode>\tAssigns <code> to <account>");
        println!("\tenv\t\t\tShow current environment");
        println!("\tenv.set.difficulty <difficulty>\t\t\tSet current difficulty");
        println!("\tenv.set.currentrandom [currentRandom]\tSet currentRandom for post-Merge transactions");
        println!("\ttxs\t\t\tShow current transactions");
        println!("\ttx.new\t\t\tCreate (empty) transaction");
        println!("\ttx.set.sender <ix> <address>\tSet <address>'s secret key in transaction with index <ix>");
        println!("\ttx.set.receiver <ix>\tSet <address> as the receiver in transaction with index <ix>"); 
        println!("\ttx.set.input <ix> <input>\t Set transaction data (<input>) in transaction with index <ix>");
        println!("\ttx.set.value <ix> <value>\t Set transaction <value> in transaction with index <ix>");
        println!("\trun\t\t\tExecute test case");
        println!("\tsave <filename>\t\tSaves current session to json file");
        println!("\tload <filename>\t\tReload previous session from json file");
        println!("\texit\t\t\tExit");
    }

    fn cmd_unknown(&self) {
        panic!("unknown command!");
    }

    fn cmd_exit(&self) -> bool{
        println!("Bye!");
        return true;
    }

    fn cmd_extract(&self, ctx: &mut Context){
        if self.command_params.len() != 1 {
            println!("Error: 1 parameter expected (state test json file)");
            return;
        }

        let c = Context::from_state_test(self.command_params[0].as_str());

        if c.is_ok() {
            let new_context = c.unwrap();
            ctx.alloc = new_context.alloc;
            ctx.env = new_context.env;
            ctx.txs = new_context.txs;
            println!("Context information extracted correctly!");
        } else {
            println!("Error, cannot open file `{}`", self.command_params[0]);
        }
    }

    fn cmd_dir(&self, ctx: &mut Context) {
        if self.command_params.len() != 1 {
            println!("Error: 1 parameter expected (state test json file)");
            return;
        }

        if ctx.set_work_dir(self.command_params[0].as_str()) {
            println!("Working directory successfully changed to {}", ctx.config.work_dir);
        } else {
            println!("Error changing working directory, check if directory exists");
        }
    }

    fn cmd_set_hard_fork(&self, ctx: &mut Context) {
        if self.command_params.len() != 1 {
            println!("Error: 1 parameter expected (HardFork name)");
            return;
        }

        ctx.config.hard_fork = self.command_params[0].clone();
        ctx.config.save();
        println!("HardFork `{}` configured!", ctx.config.hard_fork);
    }

    fn cmd_run(&self, ctx: &mut Context) {
        ctx.run();
    }

    fn cmd_set_t8n(&self, ctx: &mut Context) {
      if self.command_params.len() != 1 {
        println!("Error: 1 parameter expected (evm path)");
        return;
      }
      ctx.config.t8n = self.command_params[0].clone();
      ctx.config.save();
      println!("Configured t8n tool {}", ctx.config.t8n);
    }

    fn cmd_set_evm(&self, ctx: &mut Context) {
      if self.command_params.len() != 1 {
        println!("Error: 1 parameter expected (evm path)");
        return;
      }

      if self.command_params[0] == "default" {
        ctx.config.evm = String::from("");
      } else {
        ctx.config.evm = self.command_params[0].clone();
      }
      ctx.config.save();
      println!("Configured evm {}", ctx.config.evm);
    }

    fn cmd_env_set_difficulty(&self, ctx: &mut Context) {
        if self.command_params.len() != 1 {
            println!("Error: expected 1 parameter (difficulty)");
            return;
        }

        ctx.env.set_current_difficulty(self.command_params[0].clone());
    }

    fn cmd_env_set_current_random(&self, ctx: &mut Context) {
        if self.command_params.len() > 1 {
            println!("Error: expected 0 or 1 parameters");
            return;
        }

        let mut current_random = "0x0000000000000000000000000000000000000000000000000000000000020000".to_string();

        if self.command_params.len() == 1 {
            current_random = self.command_params[0].clone();
        }
        ctx.env.set_current_random(current_random);
    }

    fn cmd_add_account(&self, ctx: &mut Context) {
        let mut address = "0x0000000000000000000000000000000000000100";
        if self.command_params.len() == 1 {
            address = self.command_params[0].as_str();
        } 
        if self.command_params.len() > 1 {
            println!("Error: Expected a maximum of 1 parameter (address)");
            return;
        }

        if ctx.add_address(address).is_ok() {
            println!("New address added with default fields");
        } else {
            println!("Error creating new address");
        }
    }

    fn cmd_add_default_account(&self, ctx: &mut Context) {
        if ctx.add_default_address().is_ok() {
            println!("Added default address");
        } else {
            println!("Error creating default address");
        }
    }

    fn cmd_add_code(&self, ctx: &mut Context) {
        if self.command_params.len() != 2 {
            println!("Error: Expected 2 parameters (address, code)");
            return;
        }
        let address = self.command_params[0].clone();
        let code = self.command_params[1].clone();

        if ctx.alloc.contains_key(&address) {
            let mut account: Alloc = ctx.alloc.get(&address).unwrap().clone();
            if account.set_code(code).is_ok() {
                if ctx.alloc.remove(&address).is_some() {
                    ctx.alloc.insert(address, account.to_owned());
                }
            } else {
                println!("Error setting account's code");
            }
        } else {
            println!("Address {} not found!", address);
        }
    }

    fn cmd_new_tx(&self, ctx: &mut Context) {
        ctx.txs.push(TransactionT8n::default());
    }

    fn cmd_tx_set_sender(&self, ctx: &mut Context) {
        if self.check_params(2, "index, address").is_err() {
            return;
        }
        let index = self.command_params[0].clone();
        let index = index.parse::<i32>();

        if index.is_ok() {
            let index = index.unwrap();
            let address = self.command_params[1].as_str();
            if ctx.txs.len() > (index as usize) && index >= 0 {
                let pk = ctx.get_secret_key(address.to_string());

                if pk.is_some() {
                    ctx.txs[index as usize].set_private_key(pk.unwrap().as_str());
                } else {
                    println!("Account not found or does not contain private key");
                }
            } else {
                println!("Transaction not found!");
            }
        } else {
            println!("Index {} is not valid!", self.command_params[0]);
        }
    }

    fn cmd_tx_set_receiver(&self, ctx: &mut Context) {
        if self.check_params(2, "index, address").is_err() {
            return;
        }
        let index = self.command_params[0].clone();
        let index = index.parse::<i32>();

        if index.is_ok() {
            let index = index.unwrap();
            let address = self.command_params[1].as_str();
            if ctx.txs.len() > (index as usize) && index >= 0 {
                ctx.txs[index as usize].set_receiver(address);
                println!("Receiver configured!");
            } else {
                println!("Transaction not found!");
            }
        } else {
            println!("Index {} is not valid!", self.command_params[0]);
        }
    }

    fn cmd_tx_set_input(&self, ctx: &mut Context) {
        if self.check_params(2, "index, input data").is_err() {
            return;
        }

        let index = self.command_params[0].clone();
        let index = index.parse::<i32>();

        if index.is_ok() {
            let index = index.unwrap();
            let input_data = self.command_params[1].as_str();
            if ctx.txs.len() > (index as usize) && index >= 0 {
                ctx.txs[index as usize].set_input(input_data);
                println!("Transaction input configured!");
            } else {
                println!("Transaction not found!");
            }
        } else {
            println!("Index {} is not valid!", self.command_params[0]);
        }
    }

    fn cmd_tx_set_value(&self, ctx: &mut Context) {
        if self.check_params(2, "index, value").is_err() {
            return;
        }

        let index = self.command_params[0].clone();
        let index = index.parse::<i32>();

        if index.is_ok() {
            let index = index.unwrap();
            let value = self.command_params[1].as_str();
            if ctx.txs.len() > (index as usize) && index >= 0 {
                ctx.txs[index as usize].set_value(value);
                println!("Transaction value configured!");
            } else {
                println!("Transaction not found!");
            }
        } else {
            println!("Index {} is not valid!", self.command_params[0]);
        }


    }

    fn cmd_save(&self, ctx: &mut Context) {
       if self.check_params(1, "filename").is_err() {
           return
       }
       let fname = self.command_params[0].as_str();
       if ctx.save(fname).is_ok() {
           println!("Context saved {}", fname);
       } else {
           println!("Error saving context");
       }
    }

    fn cmd_load(&self, ctx: &mut Context) {
        if self.check_params(1, "filename").is_err() {
            return
        }
        let fname = self.command_params[0].as_str();
        if ctx.load(fname).is_ok() {
            println!("Context loaded correctly");
        } else {
            println!("Error loading context from file {}", fname);
        }
    }

    pub fn execute(&self, ctx : &mut Context) -> bool {
        match self.command_name.as_str() {
            "help" => self.cmd_help(),
            "extract" => self.cmd_extract(ctx),
            "dir" => self.cmd_dir(ctx),
            "alloc" => ctx.print_alloc(),
            "alloc.add" => self.cmd_add_account(ctx),
            "alloc.add.default" => self.cmd_add_default_account(ctx),
            "addcode" => self.cmd_add_code(ctx),
            "env" => ctx.print_env(),
            "env.set.difficulty" => self.cmd_env_set_difficulty(ctx),
            "env.set.currentrandom" => self.cmd_env_set_current_random(ctx),
            "txs" => ctx.print_txs(),
            "tx.new" => self.cmd_new_tx(ctx),
            "tx.set.sender" => self.cmd_tx_set_sender(ctx),
            "tx.set.receiver" => self.cmd_tx_set_receiver(ctx),
            "tx.set.input" => self.cmd_tx_set_input(ctx),
            "tx.set.value" => self.cmd_tx_set_value(ctx),
            "hf" => self.cmd_set_hard_fork(ctx),
            "t8n" => self.cmd_set_t8n(ctx),
            "evm" => self.cmd_set_evm(ctx),
            "run" => self.cmd_run(ctx),
            "save" => self.cmd_save(ctx),
            "load" => self.cmd_load(ctx),
            "exit" => return self.cmd_exit(),
            _ => self.cmd_unknown()
        }
        return false;
    }
}


pub struct Repl {
    context: Context
}

impl Repl {
    pub fn new() -> Repl {
        let context = Context::default();
        Repl {
            context
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.welcome_message();
        loop {
            let mut user_input = String::new();
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            let mut prompt = String::from(self.context.config.hard_fork.as_str());
            prompt.push_str(" > ");
            stdout.write(prompt.as_bytes());
            stdout.flush();
            stdin.read_line(&mut user_input)?;

            let command = ReplCommand::from_string(user_input);

            if command.is_some() {
                let exit = command.unwrap().execute(&mut self.context);

                if exit {
                    return Ok(())
                }
            } else {
                println!("Command not found!");
            }
        }
    }

    fn welcome_message(&self) {
        println!("Welcome to t8n-repl");
        println!("Default working directory {}", self.context.config.work_dir);
        println!("t8n tool: {} {}", self.context.config.t8n, self.context.config.evm);
    }
}


