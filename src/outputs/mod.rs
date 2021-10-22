pub mod custom;
pub mod discord_bot;
pub mod discord_webhook;
pub mod slack;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serialize;

use crate::{config::OutputConfig, Result};

use self::{
    custom::Custom, discord_bot::DiscordBot, discord_webhook::DiscordWebhook, slack::Slack,
};

pub struct Output {
    output: Box<dyn OutputTrait + Send + Sync>,
}

impl Output {
    pub fn new(output_config: OutputConfig, client: Client) -> Self {
        let output: Box<dyn OutputTrait + Send + Sync> = match output_config {
            OutputConfig::Custom {
                command,
                arguments,
                use_stdin,
            } => Box::new(Custom::new(command, arguments, use_stdin)),
            OutputConfig::DiscordWebhook { url } => Box::new(DiscordWebhook::new(url, client)),
            OutputConfig::DiscordBot { token, user_id } => {
                Box::new(DiscordBot::new(token, user_id))
            }
            OutputConfig::Slack { url } => Box::new(Slack::new(url, client)),
        };

        Self { output }
    }

    pub async fn push(&self, name: &str, entries: &[Entry]) -> Result<()> {
        self.output.push(name, entries).await
    }
}

#[async_trait]
trait OutputTrait {
    async fn push(&self, name: &str, entries: &[Entry]) -> Result<()>;
}

const TEXT_WIDTH: usize = 80;
const DESCRIPTION_MAX_LENGTH: usize = 256;

#[derive(Serialize)]
pub struct Entry {
    title: String,
    description: String,
    author: Option<String>,
    url: String,
    timestamp: DateTime<Utc>,
}

impl From<feed_rs::model::Entry> for Entry {
    fn from(entry: feed_rs::model::Entry) -> Self {
        let description = if let Some(summary) = entry.summary {
            match summary.content_type.subtype().as_str() {
                "html" => html2text::from_read(summary.content.as_bytes(), TEXT_WIDTH),
                _ => summary.content,
            }
        } else if let Some(content) = entry.content {
            match content.content_type.subtype().as_str() {
                "html" => content.body.map_or_else(
                    || "".to_owned(),
                    |body| html2text::from_read(body.as_bytes(), TEXT_WIDTH),
                ),
                _ => content.body.unwrap_or_else(|| "".to_owned()),
            }
        } else {
            "".to_owned()
        };

        let description = if description.len() > DESCRIPTION_MAX_LENGTH {
            format!(
                "{}...",
                description
                    .chars()
                    .take(DESCRIPTION_MAX_LENGTH)
                    .collect::<String>()
            )
        } else {
            description
        };

        Self {
            title: entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "".to_owned()),
            description,
            author: entry.authors.first().map(|p| p.name.clone()),
            url: entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default(),
            timestamp: entry.published.unwrap(),
        }
    }
}
