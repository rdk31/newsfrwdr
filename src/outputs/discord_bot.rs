use log::debug;

use serenity::{builder::CreateEmbed, http::Http};

use super::{Entry, OutputTrait};
use crate::Result;
use async_trait::async_trait;

pub struct DiscordBot {
    token: String,
    user_id: u64,
}

impl DiscordBot {
    pub fn new(token: String, user_id: u64) -> Self {
        Self { token, user_id }
    }
}

#[async_trait]
impl OutputTrait for DiscordBot {
    async fn push(&self, _: &str, entries: &[Entry]) -> Result<()> {
        debug!("pushing {} entries to discord bot", entries.len());

        let http = Http::new_with_token(&self.token);

        let user = http.get_user(self.user_id).await?;

        for chunk in entries.chunks(10) {
            let embeds: Vec<CreateEmbed> = chunk
                .iter()
                .map(|entry| {
                    let mut e = CreateEmbed::default();
                    e.title(entry.title.clone());
                    e.description(entry.description.clone());
                    if let Some(author) = entry.author.as_ref() {
                        e.author(|a| {
                            a.name(author.clone());
                            a
                        });
                    }
                    e.url(entry.url.clone());
                    e.timestamp(entry.timestamp.to_rfc3339());
                    e
                })
                .collect();

            user.dm(&http, |m| {
                m.set_embeds(embeds);
                m
            })
            .await?;
        }

        Ok(())
    }
}
