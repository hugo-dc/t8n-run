use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use home;

struct Command {
    command_name: String
}

impl Command {
    pub fn from_string(st : String) -> Option<Command> {
        let valid_commands = vec!["exit", "help"];

        let input_command = st.clone();
        let words: Vec<&str> = input_command.split(' ').collect();

        let cmd : String;
        if words.len() == 0 {
            cmd = st.trim().to_string(); 
        } else {
            cmd = words[0].trim().to_string();
        }

        println!("cmd: <{}>", cmd);
        if valid_commands.iter().find(|&&s| s == cmd.as_str()).is_some() {
           return Some(Command { command_name : cmd });
        }
        return None;
    }

    fn cmd_help(&self) {
        println!("this is help");
    }

    fn cmd_unknown(&self) {
        panic!("unknown command!");
    }

    fn cmd_exit(&self) -> bool{
        println!("Bye!");
        return true;
    }

    pub fn execute(&self) -> bool {
        println!("Executing <{}>", self.command_name);
        match self.command_name.as_str() {
            "help" => self.cmd_help(),
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
