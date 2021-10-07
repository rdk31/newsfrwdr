pub mod discord;

use async_trait::async_trait;
use feed_rs::model::Entry;
use reqwest::Client;

use crate::{config::OutputConfig, Result};

use discord::Discord;

pub struct Output {
    output: Box<dyn OutputTrait + Send + Sync>,
}

impl Output {
    pub fn new(output_config: OutputConfig, client: Client) -> Self {
        let output = match output_config {
            OutputConfig::Discord { url } => Box::new(Discord::new(url, client)),
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
