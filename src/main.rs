use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::io::BufReader;
use std::collections::HashMap;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use home;


// utils
fn hex_remove_leading_zero(hex: String) -> String {
  if hex.starts_with("0x0") {
    String::from("0x") + &hex[3..]
  } else {
    hex
  }
}

#[derive(Deserialize, Debug, Clone)]
struct Info {
    comment: String,
    #[serde(rename="filling-rpc-server")]
    filling_rpc_server : String,
    #[serde(alias="filling-tool-version")]
    filling_tool_version : String,
    #[serde(alias="generatedTestHash")]
    generated_test_hash : String,
    lllcversion : String,
    source: String,
    #[serde(alias="sourceHash")]
    source_hash : String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Env {
    #[serde(rename="currentBaseFee")]
    current_base_fee : String,
    #[serde(rename="currentCoinbase")]
    current_coinbase : String,
    #[serde(rename="currentDifficulty")]
    current_difficulty : String,
    #[serde(rename="currentGasLimit")]
    current_gas_limit : String,
    #[serde(rename="currentNumber")]
    current_number : String,
    #[serde(rename="currentTimestamp")]
    current_timestamp : String,
    #[serde(rename="previousHash")]
    previous_hash : String
}

impl Env {
    pub fn new() -> Env {
        Env {
            current_base_fee : String::from("0x0a"),
            current_coinbase : String::from("0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba"),
            current_difficulty : String::from("0x020000"),
            current_gas_limit : String::from("0x05f5e100"),
            current_number : String::from("0x01"),
            current_timestamp : String::from("0x03e8"),
            previous_hash : String::from("0x5e20a0453cecd065ea59c37ac63e079ee08998b6045136a8ce6635c7912ec0b6")
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Indexes {
    data : u64,
    gas : u64,
    value : u64,
}

#[derive(Debug, Deserialize, Clone)]
struct Post {
    hash : String,
    indexes: Indexes,
    logs: String,
    txbytes: String
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TransactionT8n {
    input : String,
    gas: String,
    #[serde(rename="gasPrice")]
    gas_price: String,
    nonce: String,
    to: String,
    value: String,
    v: String,
    r: String,
    s: String,
    #[serde(rename="secretKey")]
    secret_key: String,
    #[serde(rename="chainId")]
    chain_id : String,
    #[serde(rename="type")]
    tx_type: Option<String>
}

impl TransactionT8n {
  pub fn new(input: String, gas: String, gas_price: String, nonce: String, to: String, value: String, secret_key: String, tx_type: Option<String>) -> TransactionT8n {

    let gas = hex_remove_leading_zero(gas);
    let gas_price = hex_remove_leading_zero(gas_price);
    let nonce = hex_remove_leading_zero(nonce);
    let value = hex_remove_leading_zero(value);

    return TransactionT8n {
      input,
      gas,
      gas_price,
      nonce,
      to,
      value,
      v: String::from("0x0"),
      r: String::from("0x0"),
      s: String::from("0x0"),
      secret_key: secret_key,
      chain_id: String::from("0x1"),
      tx_type
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
struct Transaction {
    data : Vec<String>,
    #[serde(alias="gasLimit")]
    gas_limit : Vec<String>,
    #[serde(alias="gasPrice")]
    gas_price: String,
    nonce: String,
    #[serde(alias="secretKey")]
    secret_key: String,
    sender: String,
    to: String,
    value : Vec<String>
}

#[derive(Debug, Deserialize, Clone)]
struct StateTestContent {
    #[serde(rename="_info")]
    info : Info,
    env : Env,
    post: HashMap<String, Vec<Post>>,
    pre: HashMap<String, Alloc>,
    transaction: Transaction
}

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
        let json_file = fs::File::open(self.command_params[0].as_str());

        if json_file.is_ok() {
            let json: HashMap<String, StateTestContent>  = serde_json::from_reader(BufReader::new(json_file.unwrap())).unwrap();
            let json_clone = json.clone();
            let test_name: String = json_clone.into_keys().collect(); 
            let state_test = json[test_name.as_str()].clone();
            let transaction = state_test.transaction.clone();
            ctx.alloc = state_test.pre.clone();
            ctx.env = state_test.env.clone();

            let mut txs : Vec<TransactionT8n> = Vec::new();

            for (i, d) in transaction.data.iter().enumerate() {
                let tx_st = transaction.clone();
                let tx_ctx = TransactionT8n::new(
                  d.to_string(), 
                  transaction.gas_limit[i].clone(), 
                  tx_st.gas_price, 
                  tx_st.nonce, 
                  tx_st.to, 
                  transaction.value[i].clone(), 
                  tx_st.secret_key, 
                  None);

                txs.push(tx_ctx);
            }
            ctx.txs = txs;
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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Config {
    work_dir: String,
    t8n: String,
    evm: String,
    hard_fork: String
}

impl Config {
    fn save(&self) -> bool {
        let home_dir = home::home_dir().expect("Cannot determine HOME directory");
        let config_file_path = String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.json";
        let config_str = serde_json::to_string(&self).unwrap();
        let mut file = fs::File::create(config_file_path).unwrap();
        if file.write(config_str.as_bytes()).is_err() {
            return false;
        } else {
            return true;
        }
    }

    pub fn new() -> Config {
        let home_dir = home::home_dir().expect("Cannot determine HOME directory");
        let config_file_path = String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.json";
        let config_file = fs::File::open(config_file_path.as_str());
        if config_file.is_ok() {
            let config: Config = serde_json::from_reader(BufReader::new(config_file.unwrap())).unwrap();
            return config;
        } else {
            let home_dir = home::home_dir().expect("Cannot determine HOME directory");
            let default_working_dir = String::from(home_dir.to_str().unwrap()) + "/t8n-repl";

            let working_dir_path = Path::new(default_working_dir.as_str()); 
            if ! working_dir_path.exists() {
                fs::create_dir(working_dir_path).expect("Error creating default working directory");
            }
            let config = Config {
               work_dir : working_dir_path.to_str().unwrap().to_string(),
               t8n: String::from("/usr/bin/evm"),
               evm : String::from(""),
               hard_fork: String::from("")
            };

            config.save();
            return config;
        }
    }

    pub fn set_work_dir(&mut self, wdir_path : &str) {
        let home_dir = home::home_dir().expect("Cannot determine HOME directory");
        let config_file_path = String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.json";

        self.work_dir = wdir_path.to_string();
        println!("Writing {} as default directory", wdir_path);
        if self.save() {
            println!("Configuration saved!");
        } else {
            println!("Error saving configuration");
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Alloc {
    balance: String,
    code: String,
    nonce: String,
    storage: HashMap<String, String>
}

struct Context {
    config : Config,
    alloc : HashMap<String, Alloc>,
    env : Env,
    txs: Vec<TransactionT8n>
}

impl Context {
    pub fn default() -> Context {
        Context {
            config: Config::new(),
            alloc: HashMap::new(),
            env: Env::new(),
            txs: Vec::new()
        }
    }

    pub fn set_work_dir(&mut self, wd: &str) -> bool {
        if ! Path::new(wd).exists() {
            return false;
        }
        self.config.work_dir = wd.to_string();
        return true;
    }

    pub fn print_alloc(&self) {
        println!("{:?}", self.alloc);
    }

    pub fn print_env(&self) {
        println!("{:?}", self.env);
    }

    pub fn print_txs(&self) {
        println!("{:?}", self.txs);
    }
}

struct Repl {
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

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
