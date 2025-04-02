use std::{env, fmt::Display, fs, process};

use config::Config;

mod config;
mod layout;

fn handle_error<T: Display, R>(message: T) -> R {
    println!("\x1b[31;1merror\x1b[0m: {message}");
    process::exit(1)
}

fn main() {
    let mut args = env::args().skip(1);
    let config_file = if let Some(path) = args.next() {
        fs::read_to_string(path).unwrap_or_else(handle_error)
    } else {
        let mut config_file = dirs::config_dir().unwrap();
        config_file.push("/dyn-fetch/config.toml");
        fs::read_to_string(config_file).unwrap_or_else(handle_error)
    };
    let config: Config = toml::from_str(&config_file).unwrap_or_else(handle_error);
    layout::display(config).unwrap_or_else(handle_error);
}
