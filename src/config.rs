use std::fs;
use std::io::Write;
use std::io::BufReader;
use std::path::Path;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub work_dir: String,
    pub t8n: String,
    pub evm: String,
    pub hard_fork: String
}

impl Config {
    pub fn save(&self) -> bool {
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


