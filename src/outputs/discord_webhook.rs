use chrono::{DateTime, Utc};
use log::debug;
use reqwest::Client;

use serde::Serialize;

use super::{Entry, OutputTrait};
use crate::Result;
use async_trait::async_trait;

pub struct DiscordWebhook {
    url: String,
    client: Client,
}

impl DiscordWebhook {
    pub fn new(url: String, client: Client) -> Self {
        Self { url, client }
    }
}

#[async_trait]
impl OutputTrait for DiscordWebhook {
    async fn push(&self, _: &str, entries: &[Entry]) -> Result<()> {
        debug!("pushing {} entries to discord webhook", entries.len());

        for chunk in entries.chunks(10) {
            let embeds: Vec<EmbedObject> = chunk
                .iter()
                .map(|entry| EmbedObject {
                    title: entry.title.clone(),
                    description: entry.description.clone(),
                    author: entry.author.as_ref().map(|name| EmbedAuthor {
                        name: name.clone(),
                        url: None,
                    }),
                    url: entry.url.clone(),
                    timestamp: entry.timestamp,
                })
                .collect();

            let message = Message { embeds };

            self.client
                .post(&self.url)
                .json(&message)
                .send()
                .await?
                .error_for_status()?;
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct Message {
    embeds: Vec<EmbedObject>,
}

#[derive(Serialize)]
struct EmbedObject {
    title: String,
    description: String,
    author: Option<EmbedAuthor>,
    url: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct EmbedAuthor {
    name: String,
    url: Option<String>,
}
