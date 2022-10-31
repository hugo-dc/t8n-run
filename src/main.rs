use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::io::BufReader;
use std::collections::HashMap;
use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
use home;

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

#[derive(Deserialize, Debug, Clone)]
struct Env {
    #[serde(alias="currentBaseFee")]
    current_base_fee : String,
    #[serde(alias="currentCoinbase")]
    current_coinbase : String,
    #[serde(alias="currentDifficulty")]
    current_difficulty : String,
    #[serde(alias="currentGasLimit")]
    current_gas_limit : String,
    #[serde(alias="currentNumber")]
    current_number : String,
    #[serde(alias="currentTimestamp")]
    current_timestamp : String,
    #[serde(alias="previousHash")]
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

#[derive(Debug, Deserialize, Clone)]
struct TransactionT8n {
    input : String,
    gas: String,
    #[serde(alias="gasPrice")]
    gas_price: String,
    nonce: String,
    to: String,
    value: String,
    v: String,
    r: String,
    s: String,
    #[serde(alias="gasPrice")]
    secret_key: String,
    #[serde(alias="chainId")]
    chain_id : String,
    #[serde(alias="type")]
    tx_type: Option<String>
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

struct Command {
    command_name: String,
    command_params: Vec<String>,
}

impl Command {
    pub fn from_string(st : String) -> Option<Command> {
        let valid_commands = vec!["exit", "help", "extract", "dir", "alloc", "env", "txs", "setHardFork"];

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
           return Some(Command { command_name : cmd, command_params: params });
        }
        return None;
    }

    fn cmd_help(&self) {
        println!("\thelp\t\t\tShows this help");
        println!("\tdir <path>\t\tSets <path> as the current working directory");
        println!("\textract <test>\t\tExtract context information from Ethereum State Test");
        println!("\tsetHardFork <hf_name>\tSet HardFork");
        println!("\tt8n <t8n path>\t\tSet t8n tool path");
        println!("\tt8nFlag <t8n_flag>\tSet t8n tool flag (if needed)");
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
                let tx_ctx = TransactionT8n {
                    input: d.to_string(),
                    gas: transaction.gas_limit[i].clone(),
                    gas_price: tx_st.gas_price,
                    nonce: tx_st.nonce,
                    to: tx_st.to,
                    value: transaction.value[i].clone(),
                    v: String::from("0x0"),
                    r: String::from("0x0"),
                    s: String::from("0x0"),
                    secret_key: tx_st.secret_key,
                    chain_id: String::from("0x1"),
                    tx_type: Some(String::from("0x1"))
                };

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
            println!("Working directory successfully changed to {}", ctx.work_dir);
        } else {
            println!("Error changing working directory, check if directory exists");
        }
    }

    fn cmd_set_hard_fork(&self, ctx: &mut Context) {
        if self.command_params.len() != 1 {
            println!("Error: 1 parameter expected (HardFork name)");
            return;
        }

        ctx.set_hard_fork(self.command_params[0].as_str());
        println!("HardFork `{}` configured!", ctx.hard_fork);
    }

    pub fn execute(&self, ctx : &mut Context) -> bool {
        match self.command_name.as_str() {
            "help" => self.cmd_help(),
            "extract" => self.cmd_extract(ctx),
            "dir" => self.cmd_dir(ctx),
            "alloc" => ctx.print_alloc(),
            "env" => ctx.print_env(),
            "txs" => ctx.print_txs(),
            "setHardFork" => self.cmd_set_hard_fork(ctx),
            "exit" => return self.cmd_exit(),
            _ => self.cmd_unknown()
        }
        return false;
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Config {
    work_dir: String
}

impl Config {
    fn get_default_working_dir() -> String {
      let home_dir = home::home_dir().expect("Cannot determine HOME directory");
      return String::from(home_dir.to_str().unwrap()) + "/t8n-repl";
    }

    fn get_config_file_path() -> String {
      let home_dir = home::home_dir().expect("Cannot determine HOME directory");
      return String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.json";
    }

    fn save(cfg: Config) -> bool {
        let config_str = serde_json::to_string(&cfg).unwrap();
        let config_file_path = Config::get_config_file_path();
        let mut file = fs::File::create(config_file_path).unwrap();
        if file.write(config_str.as_bytes()).is_err() {
            return false;
        } else {
            return true;
        }
    }

    pub fn get() -> Config {
        let config_file_path = Config::get_config_file_path();
        let config_file = fs::File::open(config_file_path.as_str());
        if config_file.is_ok() {
            let config: Config = serde_json::from_reader(BufReader::new(config_file.unwrap())).unwrap();
            return config;
        } else {
            let default_working_dir = Config::get_default_working_dir();
            let working_dir_path = Path::new(default_working_dir.as_str()); 
            if ! working_dir_path.exists() {
                fs::create_dir(working_dir_path).expect("Error creating default working directory");
            }
            let config = Config {
               work_dir : working_dir_path.to_str().unwrap().to_string()
            };

            Config::save(config.clone());
            return config;
        }
    }

    pub fn set_work_dir(wdir_path : &str) {
        let home_dir = home::home_dir().expect("Cannot determine HOME directory");
        let config_file_path = String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.json";
        let mut config = Config::get();

        config.work_dir = wdir_path.to_string();
        println!("Writing {} as default directory", wdir_path);
        if Config::save(config) {
            println!("Configuration saved!");
        } else {
            println!("Error saving configuration");
        }
    }
}


#[derive(Deserialize, Debug, Clone)]
struct Alloc {
    balance: String,
    code: String,
    nonce: String,
    storage: HashMap<String, String>
}

struct Context {
    work_dir : String,
    hard_fork : String,
    alloc : HashMap<String, Alloc>,
    env : Env,
    txs: Vec<TransactionT8n>
}

impl Context {
    pub fn default() -> Context {
        let work_dir = Config::get().work_dir;
        Context {
            work_dir,
            hard_fork: String::new(),
            alloc: HashMap::new(),
            env: Env::new(),
            txs: Vec::new()
        }
    }

    pub fn set_work_dir(&mut self, wd: &str) -> bool {
        if ! Path::new(wd).exists() {
            return false;
        }
        self.work_dir = wd.to_string();
        Config::set_work_dir(self.work_dir.as_str());
        return true;
    }

    pub fn set_hard_fork(&mut self, hf: &str) {
        self.hard_fork = hf.to_string();
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
            let mut prompt = String::from(self.context.hard_fork.as_str());
            prompt.push_str(" > ");
            stdout.write(prompt.as_bytes());
            stdout.flush();
            stdin.read_line(&mut user_input)?;

            let command = Command::from_string(user_input);

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
        println!("Default working directory {}", self.context.work_dir);
    }
}

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
