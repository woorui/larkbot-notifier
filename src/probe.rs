mod larkbot;
use chrono::Local;
use larkbot::Bot;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::{self, signal, task};

use serde::Deserialize;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print!("Usage: {} <config path> \n", args[0]);
        return;
    }

    let config_file_path = &args[1];

    let configs = read_config_from_file(config_file_path);

    println!("{:?}", configs);

    let bot = match larkbot::newbot(larkbot::BotType::Unsafer) {
        Some(bot) => bot,
        None => return,
    };

    let running = Arc::new(AtomicBool::new(true));

    let running_clone = running.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        running_clone.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    for config in configs {
        if config.cmd.len() < 1 {
            continue;
        }

        let config_clone = config.clone();
        let running_clone = Arc::clone(&running);
        // clone for the loop
        let bot_clone = Arc::clone(&bot);

        tokio::spawn(async move {
            while running_clone.load(Ordering::SeqCst) {
                // clone for the spawn
                let bot_clone = Arc::clone(&bot_clone);

                run_command(bot_clone, &config_clone.name, config_clone.cmd.clone());

                std::thread::sleep(std::time::Duration::from_secs(
                    config_clone.duration.as_secs(),
                ));
            }
        });
    }

    task::spawn(async {}).await.unwrap();
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    duration: Duration,
    name: String,
    cmd: Vec<String>,
}

fn read_config_from_file(filepath: &String) -> Vec<Config> {
    let mut file =
        File::open(filepath).expect(format!("Cannot open config file: {}", filepath).as_str());

    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect(format!("Cannot read config file: {}", filepath).as_str());

    serde_yaml::from_str(&contents).unwrap()
}

fn run_command(bot: Arc<dyn Bot>, name: &String, cmd: Vec<String>) {
    if cmd.len() > 1 {
        let program = cmd[0].to_string();
        let mut command = Command::new(program);
        if cmd.len() > 2 {
            command.args(cmd[1..].to_vec());
        }
        let output = command.output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Command executed successfully");
                } else {
                    let code = output.status.code().unwrap_or(-1);

                    let stdout = output.stdout;
                    let stdoutstr = String::from_utf8_lossy(&stdout).to_string();

                    let _ = bot.send(&larkbot::Event {
                        event: name.to_string(),
                        event_time: Local::now(),
                        user: "zipper".to_string(),
                        description: format!("probe resut: code={}, stdout={}", code, stdoutstr),
                    });
                    println!("Command failed with exit code: {}", code);
                }
            }
            Err(e) => {
                let _ = bot.send(&larkbot::Event {
                    event: name.to_string(),
                    event_time: Local::now(),
                    user: "zipper".to_string(),
                    description: format!("probe failed: err={}", e),
                });
                println!("Command failed to execute, {}", e);
            }
        };
    }
}
