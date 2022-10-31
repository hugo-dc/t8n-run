use std::io;
use std::process::Command;
use std::io::Write;
use std::fs;

use crate::context::Context;

struct ReplCommand {
    command_name: String,
    command_params: Vec<String>,
}

impl ReplCommand {
    pub fn from_string(st : String) -> Option<ReplCommand> {
        let valid_commands = vec!["exit", "help", "extract", "dir", "alloc", "env", "txs", "hf", "run", "t8n", "evm"];

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

    fn cmd_help(&self) {
        println!("\thelp\t\t\tShows this help");
        println!("\tdir <path>\t\tSets <path> as the current working directory");
        println!("\textract <test>\t\tExtract context information from Ethereum State Test");
        println!("\thf <hf_name>\tSet HardFork");
        println!("\tt8n <t8n path>\t\tSet t8n tool path");
        println!("\tevm <evm_path>\tSet custom EVMC vm");
        println!("\talloc\t\t\tShow current allocation data");
        println!("\tenv\t\t\tShow current environment");
        println!("\ttxs\t\t\tShow current transactions");
        println!("\trun\t\t\tExecute test case");
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

        let mut c = Context::from_state_test(self.command_params[0].as_str());

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
      let work_dir = ctx.config.work_dir.clone();

      // Save json files
      let alloc = ctx.alloc.clone();
      let alloc_str = serde_json::to_string(&alloc).unwrap();
      let alloc_file_path = work_dir.clone() + "/alloc.json";
      let mut file = fs::File::create(alloc_file_path).unwrap();
      file.write(alloc_str.as_bytes()).expect("Error writing alloc.json file");


      let env = ctx.env.clone();
      let env_str = serde_json::to_string(&env).unwrap();
      let env_file_path = work_dir.clone() + "/env.json";
      let mut file = fs::File::create(env_file_path).unwrap();
      file.write(env_str.as_bytes()).expect("Error writing env.json file");

      let txs = ctx.txs.clone();
      let txs_str = serde_json::to_string(&txs).unwrap();
      let txs_file_path = work_dir.clone() + "/txs.json";
      let mut file = fs::File::create(txs_file_path).unwrap();
      file.write(txs_str.as_bytes()).expect("Error writing txs.json file");

      // Execute t8n tool
      let fork_flag = String::from("--state.fork=") + ctx.config.hard_fork.as_str();
      let alloc_flag = String::from("--input.alloc=") + work_dir.as_str() + "/alloc.json";
      let env_flag = String::from("--input.env=") + work_dir.as_str() + "/env.json";
      let txs_flag = String::from("--input.txs=") + work_dir.as_str() + "/txs.json";
      let result_flag = String::from("--output.result=") + "alloc_jsontx.json";
      let body_flag = String::from("--output.body=") + "/signed_txs.rlp";
      let basedir_flag = String::from("--output.basedir=") + work_dir.as_str();
      let trace_flag = String::from("--trace");

      let mut args: Vec<String> = Vec::new();

      if ctx.config.evm.as_str() != "" {
        let vm_flag = String::from("--vm.evm=") + ctx.config.evm.as_str();
        args.push(vm_flag);
      }

      args.push("t8n".to_string());
      args.push(fork_flag);
      args.push(alloc_flag);
      args.push(env_flag);
      args.push(txs_flag);
      args.push(result_flag);
      args.push(body_flag);
      args.push(basedir_flag);
      args.push(trace_flag);

      let mut cmd = Command::new(ctx.config.t8n.clone());
      cmd.args(args);

      let output = cmd.output().unwrap();
      let output_text = String::from_utf8(output.stdout).unwrap();
      let error_text = String::from_utf8(output.stderr).unwrap();
      println!("{}", output_text);
      println!("{}", error_text);

      // TODO: Read trace output
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

    pub fn execute(&self, ctx : &mut Context) -> bool {
        match self.command_name.as_str() {
            "help" => self.cmd_help(),
            "extract" => self.cmd_extract(ctx),
            "dir" => self.cmd_dir(ctx),
            "alloc" => ctx.print_alloc(),
            "env" => ctx.print_env(),
            "txs" => ctx.print_txs(),
            "hf" => self.cmd_set_hard_fork(ctx),
            "t8n" => self.cmd_set_t8n(ctx),
            "evm" => self.cmd_set_evm(ctx),
            "run" => self.cmd_run(ctx),
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


