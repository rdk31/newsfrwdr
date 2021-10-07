use chrono::{DateTime, Utc};
use feed_rs::model::Entry;
use log::debug;
use reqwest::Client;

use serde::Serialize;

use super::OutputTrait;
use crate::Result;
use async_trait::async_trait;

pub struct Discord {
    url: String,
    client: Client,
}

impl Discord {
    pub fn new(url: String, client: Client) -> Self {
        Self { url, client }
    }
}

#[async_trait]
impl OutputTrait for Discord {
    async fn push(&self, name: &str, entries: &[&Entry]) -> Result<()> {
        for chunk in entries.chunks(10) {
            let embeds: Vec<EmbedObject> = chunk
                .iter()
                .map(|&entry| EmbedObject::new(name, entry.clone()))
                .collect();

            debug!("pushing {} embeds to discord webhook", embeds.len());

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
    url: String,
    timestamp: DateTime<Utc>,
}

impl EmbedObject {
    fn new(name: &str, entry: Entry) -> Self {
        let link = entry.links.first().unwrap();

        Self {
            title: format!("{} - {}", name, entry.title.unwrap().content),
            url: link.href.clone(),
            timestamp: entry.published.unwrap(),
        }
    }
}
