mod larkbot;
use chrono::Local;
use futures::future;
use larkbot::Bot;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::interval;

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

    let task_list :Vec<JoinHandle<()>>= configs
        .into_iter()
        .filter(|config| config.cmd.len() > 1)
        .map(|config| {
            let bot = Arc::clone(&bot);

            return tokio::spawn(async move {
                // clone for the loop
                let config_clone = config.clone();

                let mut ticker = interval(Duration::from_secs(config_clone.duration));

                loop {
                    let bot_clone = Arc::clone(&bot);

                    tokio::select! {
                        _ = ticker.tick() => {
                            run_command(bot_clone, &config_clone.name, config_clone.cmd.clone()).await;
                        }
                        _ = tokio::signal::ctrl_c() => {
                            break;
                        }
                    };
                };
            })
        })
        .collect();

    future::join_all(task_list).await;

    println!("probe exit");
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    duration: u64,
    name: String,
    cmd: Vec<String>,
}

fn read_config_from_file(filepath: &String) -> Vec<Config> {
    let mut file =
        File::open(filepath).expect(format!("cannot open config file: {}", filepath).as_str());

    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect(format!("cannot read config file: {}", filepath).as_str());

    match serde_yaml::from_str::<Vec<Config>>(&contents) {
        Ok(value) => value,
        Err(error) => {
            println!("cannot parse config: filepath={}, err={}", filepath, error);
            Vec::new()
        }
    }
}

async fn run_command(bot: Arc<dyn Bot + Sync + Send>, name: &String, cmd: Vec<String>) {
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
                    println!("command `{}` successfully", cmd.join(" "));
                } else {
                    let code = output.status.code().unwrap_or(-1);

                    let stderr = output.stderr;
                    let stderr_string = String::from_utf8_lossy(&stderr).to_string();

                    let larkbot_result = bot
                        .send(&larkbot::Event {
                            event: name.to_string(),
                            event_time: Local::now(),
                            user: cmd.join(" "),
                            description: format!(
                                "probe resut: code={}, stdout={}",
                                code, stderr_string
                            ),
                        })
                        .await;

                    println!(
                        "command `{}` failed with code: {}, larkmsg={}",
                        cmd.join(" "),
                        code,
                        larkbot_result.msg
                    );
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
