use std::{env, fmt::Display, fs, io::stdout, path::PathBuf, process, str};

use config::{ArtPosition, Config};

mod config;
mod layout;

fn handle_error<T: Display, R>(message: T) -> R {
    println!("\x1b[31;1merror\x1b[0m: {message}");
    process::exit(1)
}

fn main() {
    let mut config_dir;
    let mut args = env::args().skip(1);
    let config_file = if let Some(path) = args.next() {
        config_dir = PathBuf::from(&path);
        config_dir.pop();

        fs::read_to_string(path).unwrap_or_else(handle_error)
    } else {
        config_dir = PathBuf::from(dirs::config_dir().unwrap());
        config_dir.push("/dyn-fetch");

        let mut config_file = config_dir.clone();
        config_file.push("/config.toml");
        fs::read_to_string(config_file).unwrap_or_else(handle_error)
    };
    let config: Config = toml::from_str(&config_file).unwrap_or_else(handle_error);
    match config.art_layout {
        ArtPosition::None => {
            config
                .layout
                .display(config.info, stdout())
                .unwrap_or_else(handle_error);
        }
        _ => {
            let mut file = config_dir.clone();
            file.push(config.art_path);
            let file = fs::read_to_string(file).unwrap_or_else(handle_error);

            match config.art_layout {
                ArtPosition::Top => {
                    println!("{}", file);
                    config
                        .layout
                        .display(config.info, stdout())
                        .unwrap_or_else(handle_error);
                }
                ArtPosition::Left => {
                    let mut buf = Vec::new();
                    config
                        .layout
                        .display(config.info, &mut buf)
                        .unwrap_or_else(handle_error);
                    let art_size = file.lines().map(|l| l.len()).max().unwrap_or(0);
                    let mut fetch_lines = buf.split(|c| *c == b'\n');
                    let mut art_lines = file.lines();
                    for (art, fetch) in art_lines.by_ref().zip(fetch_lines.by_ref()) {
                        println!(
                            "{art}{} {fetch}",
                            " ".repeat(art_size - art.len()),
                            fetch = str::from_utf8(fetch).unwrap()
                        );
                    }
                    for line in art_lines {
                        println!("{line}");
                    }
                    for line in fetch_lines {
                        println!("{} {}", " ".repeat(art_size), str::from_utf8(line).unwrap());
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
