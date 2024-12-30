use crate::{
    state::State,
    tempo::{TempoDay, TempoDayState},
};
use reqwest::Client;
use std::{collections::HashMap, time::Duration};
use tokio::time::timeout;

#[derive(Debug)]
pub struct TGBot {
    chat_id: String,
    client: Client,
    post_url: String,
}

impl TGBot {
    pub fn new(bot_token: &str, chat_id: &str) -> Self {
        Self {
            chat_id: chat_id.to_owned(),
            client: reqwest::Client::new(),
            post_url: format!("https://api.telegram.org/bot{}/sendMessage", bot_token),
        }
    }

    pub async fn send_message(&self, text: &str) {
        let mut map = HashMap::new();
        map.insert("chat_id", self.chat_id.to_owned());
        map.insert("text", text.to_owned());

        let _ = timeout(
            Duration::from_secs(5),
            self.client.post(self.post_url.as_str()).json(&map).send(),
        )
        .await;
    }

    pub fn send_message_mock(&self, text: &str) {
        println!("TGBot : {}", text);
    }

    pub async fn handle_day(&self, today: Option<TempoDay>, tomorrow: Option<TempoDay>) {
        let mut msg = String::new();

        if let Some(today) = today {
            msg.push_str(today.to_french_complete_string().as_str());
            msg.push_str("\n");
        }

        if let Some(tomorrow) = tomorrow {
            msg.push_str(tomorrow.to_french_complete_string().as_str());
        }

        self.send_message(msg.as_str()).await;
    }
}
