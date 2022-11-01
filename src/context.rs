use std::collections::HashMap;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::config::Config;

// utils
fn hex_remove_leading_zero(hex: String) -> String {
  if hex.starts_with("0x0") {
    String::from("0x") + &hex[3..]
  } else {
    hex
  }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionT8n {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alloc {
    balance: String,
    code: String,
    nonce: String,
    storage: HashMap<String, String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Env {
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

pub struct Context {
    pub config : Config,
    pub alloc : HashMap<String, Alloc>,
    pub env : Env,
    pub txs: Vec<TransactionT8n>
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

    pub fn from_state_test(st_name: &str) -> Result<Context, &'static str> {
        let mut ctx = Context::default();
        let json_file = fs::File::open(st_name);

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
            return Ok(ctx);
        } else {
            return Err("Error opening state test");
        }
    }

    pub fn set_work_dir(&mut self, wd: &str) -> bool {
        if ! Path::new(wd).exists() {
            return false;
        }
        self.config.work_dir = wd.to_string();
        return true;
    }

    pub fn run(&self) {
      let work_dir = &self.config.work_dir.clone();

      // Save json files
      let alloc = &self.alloc.clone();
      let alloc_str = serde_json::to_string(&alloc).unwrap();
      let alloc_file_path = work_dir.clone() + "/alloc.json";
      let mut file = fs::File::create(alloc_file_path).unwrap();
      file.write(alloc_str.as_bytes()).expect("Error writing alloc.json file");


      let env = &self.env.clone();
      let env_str = serde_json::to_string(&env).unwrap();
      let env_file_path = work_dir.clone() + "/env.json";
      let mut file = fs::File::create(env_file_path).unwrap();
      file.write(env_str.as_bytes()).expect("Error writing env.json file");

      let txs = &self.txs.clone();
      let txs_str = serde_json::to_string(&txs).unwrap();
      let txs_file_path = work_dir.clone() + "/txs.json";
      let mut file = fs::File::create(txs_file_path).unwrap();
      file.write(txs_str.as_bytes()).expect("Error writing txs.json file");

      // Execute t8n tool
      let fork_flag = String::from("--state.fork=") + &self.config.hard_fork.as_str();
      let alloc_flag = String::from("--input.alloc=") + work_dir.as_str() + "/alloc.json";
      let env_flag = String::from("--input.env=") + work_dir.as_str() + "/env.json";
      let txs_flag = String::from("--input.txs=") + work_dir.as_str() + "/txs.json";
      let result_flag = String::from("--output.result=") + "alloc_jsontx.json";
      let body_flag = String::from("--output.body=") + "/signed_txs.rlp";
      let basedir_flag = String::from("--output.basedir=") + work_dir.as_str();
      let trace_flag = String::from("--trace");

      let mut args: Vec<String> = Vec::new();

      if &self.config.evm.as_str() != &"" {
        let vm_flag = String::from("--vm.evm=") + &self.config.evm.as_str();
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

      let mut cmd = Command::new(&self.config.t8n.clone());
      cmd.args(args.clone());

      let output = cmd.output().unwrap();
      let output_text = String::from_utf8(output.stdout).unwrap();
      let error_text = String::from_utf8(output.stderr).unwrap();
      println!("{}", output_text);
      println!("{}", error_text);

      // TODO: Read trace output
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

#[derive(Debug, Deserialize, Clone)]
struct StateTestContent {
    #[serde(rename="_info")]
    info : Info,
    env : Env,
    post: HashMap<String, Vec<Post>>,
    pre: HashMap<String, Alloc>,
    transaction: Transaction
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

#[derive(Debug, Deserialize, Clone)]
struct Post {
    hash : String,
    indexes: Indexes,
    logs: String,
    txbytes: String
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
struct Indexes {
    data : u64,
    gas : u64,
    value : u64,
}


