use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;
use crate::consts;
use crate::applications;

/// Main tag to update if new applications are going to be added.
pub fn interpret_args(arg_string: &Vec<String>) {
    if arg_string.len() < 2 {
        return;
    }
    let app_mnemonic = arg_string[1].to_uppercase(); // Make whatever was put in case-insensitive
    let mut app_map = ApplicationMap::new();
    app_map.add_standard_apps();
    app_map.execute_app_from_mnem(app_mnemonic, arg_string);
}

struct ApplicationMap {
    app_list: HashMap<String, Application>,
}

impl ApplicationMap {
    /// Creates a completely empty map.
    pub fn new() -> Self {
        Self {
            app_list: HashMap::new(),
        }
    }

    pub fn add_standard_apps(&mut self) {
        // Assembler
        let assembler_title = "Hack Assembler".to_string();
        let assembler_help = "This takes in a file with Hack Assembly language and converts it directly into binary format.".to_string();
        let assembler_app = Application::new(assembler_title,assembler_help,applications::assembler::do_assembly);
        self.add_app(consts::ASSEMBLER_APP.to_string(), assembler_app);
    }

    pub fn add_app(&mut self, key: String, app: Application) {
        self.app_list.insert(key, app);
    }

    pub fn execute_app_from_mnem(self, mnem: String, args: &Vec<String>) {
        match self.app_list.get(&mnem) {
            Some(app) => {
                app.exec_app(args);
            }

            _ => {
                println!("Unrecognized command.");
            }
        }
    }
}

struct Application {
    title: String,
    help_text: String,
    fnptr: fn(&Vec<String>),
}

impl Application {
    /// Creates a new application, defined by a struct.
    pub fn new(title: String, help_text: String, fnptr: fn(&Vec<String>)) -> Self {
        Self {
            title,
            help_text,
            fnptr,
        }
    }

    /// Executes an application given arguments. The function pointer that represents the entry point into the application must take a data structure like "args" as the parameter.
    pub fn exec_app(&self, args: &Vec<String>) {
        let ptr = self.fnptr;
        ptr(args);
    }
}
/// Handles a file path and returns a file ready to manipulate in the context where this function is called. The second parameter, if set to true, indicates a write rather
/// than just a read.
pub fn handle_file_path(file_path: &str, for_write: bool) -> Result<File, &str> {
    let fpath = Path::new(file_path);
    if for_write {
        match File::create(fpath) {
            Ok(file) => Ok(file),
            Err(_) => Err(consts::FILE_CREATE_ERROR),
        }
    } else {
        match File::open(fpath) {
            Ok(file) => Ok(file),
            Err(_) => Err(consts::FILE_OPEN_ERROR),
        }
    }
}

/// Simple function to collect environment arguments to drive this utility.
pub fn collect_env_args() -> Vec<String> {
    env::args().collect()
}