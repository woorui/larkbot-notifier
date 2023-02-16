use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, Url};

use super::{parse_to_lark_request, Bot, Event, LarkBotResult};

#[derive(Clone)]
pub struct Unsafer {
    client: Client,
    url: Url,
}

impl Unsafer {
    pub fn new(bot_url: &str) -> Unsafer {
        return Unsafer {
            client: Client::new(),
            url: Url::parse(bot_url).unwrap(),
        };
    }

    async fn request(self: &Unsafer, event: &Event) -> Result<LarkBotResult> {
        let result: LarkBotResult = self
            .client
            .post(self.url.clone())
            .json(&parse_to_lark_request(event))
            .send()
            .await?
            .json()
            .await?;

        Ok(result)
    }
}

#[async_trait]
impl Bot for Unsafer {
    // TODO: be sharing rather than clone.
    fn clone_box(&self) -> Box<dyn Bot + Sync + Send> {
        Box::new(Unsafer {
            client: self.client.clone(),
            url: self.url.clone(),
        })
    }

    async fn send(self: &Unsafer, event: &Event) -> LarkBotResult {
        match self.request(event).await {
            Ok(result) => result,
            Err(err) => LarkBotResult {
                code: -1,
                msg: err.to_string(),
                data: serde_json::Value::Null,
            },
        }
    }
}
