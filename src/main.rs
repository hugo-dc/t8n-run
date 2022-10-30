use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::io::BufReader;
use std::collections::HashMap;
use serde_json::{Result, Value};
use serde::Deserialize;
use home;

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Debug, Deserialize)]
struct Indexes {
    data : u64,
    gas : u64,
    value : u64,
}

#[derive(Debug, Deserialize)]
struct Post {
    hash : String,
    indexes: Indexes,
    logs: String,
    txbytes: String
}

#[derive(Debug, Deserialize)]
struct Pre {
    balance: String,
    code: String,
    nonce: String,
    storage: HashMap<String, String>
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct StateTestContent {
    #[serde(rename="_info")]
    info : Info,
    env : Env,
    post: HashMap<String, Vec<Post>>,
    pre: HashMap<String, Pre>,
    transaction: Transaction
}

struct Command {
    command_name: String,
    command_params: Vec<String>,
}

impl Command {
    pub fn from_string(st : String) -> Option<Command> {
        let valid_commands = vec!["exit", "help", "extract"];

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
        println!("\textract <test>\t\tExtract context information from Ethereum State Test");
        println!("\texit\t\t\tExit");
    }

    fn cmd_unknown(&self) {
        panic!("unknown command!");
    }

    fn cmd_exit(&self) -> bool{
        println!("Bye!");
        return true;
    }

    fn cmd_extract(&self){
        if self.command_params.len() != 1 {
            println!("Error: 1 parameter expected (state test json file)");
            return;
        }
        let json_file = fs::File::open(self.command_params[0].as_str());

        if json_file.is_ok() {
            let json: HashMap<String, StateTestContent>  = serde_json::from_reader(BufReader::new(json_file.unwrap())).unwrap();
            println!("{:?}", json);
        } else {
            println!("Error, cannot open file `{}`", self.command_params[0]);
        }
    }

    pub fn execute(&self) -> bool {
        println!("Executing <{}>", self.command_name);
        match self.command_name.as_str() {
            "help" => self.cmd_help(),
            "extract" => self.cmd_extract(),
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
}

struct Context {
    work_dir : String
}

impl Context {
    pub fn default() -> Context {
        let work_dir = Config::get();
        Context {
            work_dir
        }
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

    pub fn run(&self) -> io::Result<()> {
        self.welcome_message();
        loop {
            let mut user_input = String::new();
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            stdout.write(String::from("> ").as_bytes());
            stdout.flush();
            stdin.read_line(&mut user_input)?;

            let command = Command::from_string(user_input);

            if command.is_some() {
                let exit = command.unwrap().execute();

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
    let repl = Repl::new();
    repl.run();
}
