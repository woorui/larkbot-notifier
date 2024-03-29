pub mod unsafer;

use std::{env, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json;

use unsafer::Unsafer;

#[async_trait]
pub trait Bot {
    async fn send(&self, event: &Event) -> LarkBotResult;
}

pub enum BotType {
    Unsafer,
}

pub fn newbot(types: BotType) -> Option<Arc<(dyn Bot + Sync + Send)>> {
    match types {
        BotType::Unsafer => match env::var("BOT_URL") {
            Ok(url) => Some(Arc::new(Unsafer::new(&url))),
            Err(err) => {
                println!("{}, `BOT_URL`", err.to_string());
                None
            }
        },
    }
}

#[derive(Deserialize, Debug)]
pub struct Event {
    pub event: String,
    pub event_time: DateTime<Local>,
    pub user: String,
    pub description: String,
}

#[derive(Deserialize, Serialize)]
pub struct LarkBotResult {
    pub code: i32,
    pub msg: String,
    pub data: serde_json::Value,
}

fn parse_to_lark_request(event: &Event) -> serde_json::Value {
    serde_json::json!({
        "msg_type": "post",
        "content": {
            "post": {
                "zh_cn": {
                    "title": format!("【NOTICE】{}", event.event),
                    "content": [
                        [{
                            "tag": "text",
                            "text": event.user,
                        }],
                        [{
                            "tag": "text",
                            "text": event.description,
                        }],
                        [{
                            "tag": "text",
                            "text": event.event_time.format("%Y-%m-%d %H:%M").to_string(),
                        }]
                    ]
                }
            }
        }
    })
}
