use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::io::BufReader;
use std::collections::HashMap;
use serde_json::{Result, Value};
use serde::Deserialize;
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
        let valid_commands = vec!["exit", "help", "extract", "dir", "alloc", "setHardFork"];

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

        println!("cmd: <{}>", cmd);
        println!("params: <{}>", params.len());
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
        println!("\talloc\t\t\tShow current allocation data");
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

    fn cmd_alloc(&self, ctx: &mut Context) {
        ctx.print_alloc();
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
        println!("Executing <{}>", self.command_name);
        match self.command_name.as_str() {
            "help" => self.cmd_help(),
            "extract" => self.cmd_extract(ctx),
            "dir" => self.cmd_dir(ctx),
            "alloc" => self.cmd_alloc(ctx),
            "setHardFork" => self.cmd_set_hard_fork(ctx),
            "exit" => return self.cmd_exit(),
            _ => self.cmd_unknown()
        }
        return false;
    }
}

struct Config {
    config_path: String
}

impl Config {
    pub fn get() -> String {
        let home_dir = home::home_dir().expect("Cannot determine HOME directory");
        let config_file_path = String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.conf";
        
        let config = fs::read_to_string(config_file_path.clone());
        if config.is_ok() {
            return config.unwrap();
        } else {
            let default_working_dir = String::from(home_dir.to_str().unwrap()) + "/t8n-repl";
            let working_dir_path = Path::new(default_working_dir.as_str());

            if ! working_dir_path.exists() {
                fs::create_dir(working_dir_path).expect("Error creating default working directory");
            }
            let mut file = fs::File::create(config_file_path).unwrap();
            file.write(default_working_dir.as_bytes());


            return default_working_dir;
        }
    }

    pub fn set(cfg_path : &str) {
        let home_dir = home::home_dir().expect("Cannot determine HOME directory");
        let config_file_path = String::from(home_dir.to_str().unwrap()) + "/.t8n-repl.conf";
        let mut file = fs::File::create(config_file_path).unwrap();
        println!("Writing {} as default directory", cfg_path);
        let result = file.write(cfg_path.as_bytes());
        println!("result: {}", result.unwrap());
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
        let work_dir = Config::get();
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
        Config::set(self.work_dir.as_str());
        return true;
    }

    pub fn set_hard_fork(&mut self, hf: &str) {
        self.hard_fork = hf.to_string();
    }

    pub fn print_alloc(&mut self) {
        println!("{:?}", self.alloc);
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
        println!("Loaded default account ");
    }
}

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
