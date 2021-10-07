use chrono::{DateTime, Utc};
use feed_rs::{
    model::{Entry, Feed},
    parser,
};
use reqwest::Client;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;

use log::{debug, error};

use crate::{config::InputConfig, error::Error, outputs::Output, Result};

const DEFAULT_INTERVAL: Duration = Duration::from_secs(60 * 30);

pub struct Input {
    name: String,

    url: String,
    interval: Duration,
    retry_limit: usize,
    retries_left: usize,
    last_date: Option<DateTime<Utc>>,

    client: Client,

    outputs: Vec<Output>,
}

impl Input {
    pub fn new(name: String, config: InputConfig, outputs: Vec<Output>, client: Client) -> Self {
        Self {
            name,

            url: config.url,
            interval: config.interval.unwrap_or(DEFAULT_INTERVAL),
            retries_left: config.retry_limit,
            retry_limit: config.retry_limit,
            last_date: None,
            client,
            outputs,
        }
    }

    async fn fetch(&self) -> Result<Feed> {
        let res = self.client.get(&self.url).send().await?;
        let body = res.error_for_status()?.bytes().await?;

        let feed = parser::parse(&body[..])?;

        Ok(feed)
    }

    pub async fn watch(mut self, mut kill: Receiver<()>) -> Result<()> {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            tokio::select! {
                biased;
                _ = kill.recv() => break,
                _ = interval.tick() => {},
            };

            let feed = match self.fetch().await {
                Ok(c) => c,
                Err(e) => {
                    if is_retriable(&e) && self.retries_left > 0 {
                        error!("error while getting items: {}", e);
                        self.retries_left -= 1;
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };

            let items = feed.entries;

            if items.is_empty() {
                continue;
            }

            let last_entry = items.first().unwrap();

            if self.last_date.is_none() {
                self.last_date = last_entry.published;
            }

            let new_entries: Vec<&Entry> = items
                .iter()
                .take_while(|e| e.published.gt(&self.last_date))
                .collect();

            #[cfg(debug_assertions)]
            let new_entries = {
                use std::env;

                if let Ok(mode) = env::var("TEST_MODE") {
                    if mode == "1" {
                        items.iter().take(3).collect()
                    } else {
                        new_entries
                    }
                } else {
                    new_entries
                }
            };

            if new_entries.is_empty() {
                continue;
            }

            debug!(
                "pushing {} items from \"{}\" feed",
                new_entries.len(),
                feed.title.unwrap().content,
            );

            for output in self.outputs.iter() {
                output.push(&self.name, &new_entries).await?;
            }

            self.last_date = last_entry.published;

            if self.retries_left != self.retry_limit {
                self.retries_left = self.retry_limit;
            }
        }

        Ok(())
    }
}

fn is_retriable(err: &Error) -> bool {
    match err {
        Error::Request(e) => e.is_timeout() || e.is_connect(),
        _ => false,
    }
}
