pub mod custom;
pub mod discord_bot;
pub mod discord_webhook;

use async_trait::async_trait;
use feed_rs::model::Entry;
use reqwest::Client;

use crate::{config::OutputConfig, Result};

use self::{custom::Custom, discord_bot::DiscordBot, discord_webhook::DiscordWebhook};

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
        };

        Self { output }
    }

    pub async fn push(&self, name: &str, entries: &[&Entry]) -> Result<()> {
        self.output.push(name, entries).await
    }
}

#[async_trait]
trait OutputTrait {
    async fn push(&self, name: &str, entries: &[&Entry]) -> Result<()>;
}
