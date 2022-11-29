use std::collections::HashMap;
use std::io::BufReader;
use std::io::Write;
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::config::Config;

// utils
fn hex_remove_leading_zero(hex: String) -> String {
  if hex.starts_with("0x0") && hex.len() > 3 {
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
    to: Option<String>,
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
      to: None,
      value,
      v: String::from("0x0"),
      r: String::from("0x0"),
      s: String::from("0x0"),
      secret_key: secret_key,
      chain_id: String::from("0x1"),
      tx_type
    }
  }
  pub fn default() -> TransactionT8n {
      TransactionT8n {
          input      : "".to_string(),
          gas        : "0xaae60".to_string(),
          gas_price  : "0xa".to_string(),
          nonce      : "0x0".to_string(),
          to         : None,
          value      : "0x0".to_string(),
          v          : "0x0".to_string(),
          r          : "0x0".to_string(),
          s          : "0x0".to_string(),
          secret_key : "".to_string(),
          chain_id   : "0x1".to_string(),
          tx_type    : Some("0x1".to_string())
      }
  }

  pub fn set_private_key(&mut self, pk: &str) {
    self.secret_key = pk.to_string(); 
  }

  pub fn set_receiver(&mut self, address: &str) {
    self.to = Some(address.to_string());
  }

  pub fn set_input(&mut self, input_data: &str) {
    self.input = input_data.to_string();
  }

  pub fn set_value(&mut self, value: &str) {
    self.value = value.to_string();
  }

  pub fn set_gas(&mut self, gas: &str) {
    self.gas = gas.to_string(); 
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Alloc {
    balance: String,
    code: String,
    nonce: String,
    storage: HashMap<String, String>,
    #[serde(skip_serializing)]
    secret_key: Option<String>
}

impl Alloc {
    pub fn default() -> Alloc {
        return Alloc {
            balance: "0x0".to_string(),
            code: "0x".to_string(),
            nonce: "0x0".to_string(),
            storage: HashMap::new(),
            secret_key: None
        }
    }
    pub fn set_code(&mut self, code: String) -> Result<(), &'static str> {
        self.code = code; 
        return Ok(())
    }
    
    pub fn get_secret_key(&self) -> Option<String> {
        return self.secret_key.clone();
    }
}

#[serde_with::skip_serializing_none]
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
    #[serde(rename="currentRandom")]
    current_random : Option<String>,
    #[serde(rename="previousHash")]
    previous_hash : String
}

impl Env {
    pub fn default() -> Env {
        Env {
            current_base_fee : String::from("0x0a"),
            current_coinbase : String::from("0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba"),
            current_difficulty : String::from("0"),
            current_gas_limit : String::from("0x05f5e100"),
            current_number : String::from("0x01"),
            current_timestamp : String::from("0x03e8"),
            current_random : Some(String::from("0x0000000000000000000000000000000000000000000000000000000000020000")),
            previous_hash : String::from("0x5e20a0453cecd065ea59c37ac63e079ee08998b6045136a8ce6635c7912ec0b6")
        }
    }

    pub fn set_current_random(&mut self, cr: String) {
        self.current_random = Some(cr);
    }

    pub fn set_current_difficulty(&mut self, diff: String) {
        self.current_difficulty = diff;
    }
}

#[derive(Deserialize)]
pub struct Trace {

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub config : Config,
    pub alloc : HashMap<String, Alloc>,
    pub env : Env,
    pub txs: Vec<TransactionT8n>
}

impl Context {
    pub fn default() -> Context {
        Context {
            config: Config::default(),
            alloc: HashMap::new(),
            env: Env::default(),
            txs: Vec::new()
        }
    }

    pub fn add_address(&mut self, address: &str) -> Result<(), &'static str> {
        self.alloc.insert(address.to_string(),
                          Alloc::default());

        Ok(())
    }

    pub fn add_default_address(&mut self) -> Result<(), &'static str> {
        let default_alloc = Alloc {
            balance: "0x3b9aca00".to_string(),
            code: "".to_string(),
            nonce: "0x0".to_string(),
            storage: HashMap::new(),
            secret_key: Some("0x45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8".to_string())
        };

        self.alloc.insert("0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b".to_string(), 
            default_alloc);
        Ok(())
    }

    pub fn address_exists(&self, address: &str) -> bool {
        self.alloc.get(address).is_some()
    }

    pub fn get_secret_key(&mut self, address: String) -> Option<String> {
        let alloc = self.alloc.get(address.as_str());

        if alloc.is_some() {
            return alloc.unwrap().get_secret_key()
        } else {
            return None
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
                let nonce = tx_st.nonce.clone();
                let nonce = &nonce[2..];
                let mut nonce = i64::from_str_radix(nonce, 16).unwrap();
                nonce = nonce + (i as i64);
                let nonce = format!("0x{:x}", nonce);
                let nonce = if nonce.as_str() == "0x" {
                    String::from("0x0")
                } else {
                    nonce
                };
                let gas_limit = if transaction.gas_limit.len() <= i {
                    transaction.gas_limit.last().unwrap().clone()
                  } else {
                    transaction.gas_limit[i].clone() 
                  };
                let tx_value = if transaction.value.len() <= i {
                    transaction.value.last().unwrap().clone()
                } else {
                    transaction.value[i].clone()
                };

                let tx_ctx = TransactionT8n::new(
                  d.to_string(), 
                  gas_limit,
                  tx_st.gas_price, 
                  nonce,
                  tx_st.to, 
                  tx_value,
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

    pub fn save(&self, fname: &str) -> Result<(), ()> {
        let file_result = fs::File::create(fname); 

        if file_result.is_err() {
            return Err(())
        }

        let mut file = file_result.unwrap();
        let ctx_str = serde_json::to_string(&self).unwrap();
        if file.write(ctx_str.as_bytes()).is_ok() {
            return Ok(())
        } else {
            return Err(())
        }
    }

    pub fn load(&mut self, fname: &str) -> Result<(), ()> {
        let file = fs::File::open(fname);

        if file.is_err() {
            return Err(())
        }

        let mut context: Result<Context, serde_json::Error> = serde_json::from_reader(BufReader::new(file.unwrap()));

        if context.is_ok() {
            let context = context.unwrap();
            self.config = context.config;
            self.alloc = context.alloc;
            self.env = context.env;
            self.txs = context.txs;
            return Ok(())
        } else {
            return Err(())
        }
    }

    pub fn run(&self) {
      let work_dir = &self.config.work_dir.clone();

      // Delete previous executions (trace-*)
      for entry in fs::read_dir(work_dir).unwrap() {
          let ent = entry.unwrap();
          let path = ent.path();
          let fname = path.file_name().unwrap().to_str().unwrap();

          if fname.starts_with("trace-") {
              if fs::remove_file(path.to_str().unwrap()).is_err() {
                  println!("Error: failed to remove previous trace {}", fname);
              }
          }
      }

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

      let traces: Vec<Trace> = Vec::new();
      for entry in fs::read_dir(work_dir).unwrap() {
          let ent = entry.unwrap();
          let path = ent.path();
          let fname = path.file_name().unwrap().to_str().unwrap();
          if fname.to_string().starts_with("trace-") {
              let trace_file = path.to_str().unwrap();
              let trace_file = fs::read_to_string(trace_file);

              if trace_file.is_ok() {
                  let contents = trace_file.unwrap();
                  let lines = contents.split('\n');

                  for line in lines {
                      println!("{}", line);
                  }
              } else {
                  println!("Error reading trace file {}", fname)
              }

          }
      }
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


