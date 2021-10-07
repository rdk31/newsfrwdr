use feed_rs::model::Entry;
use log::debug;

use serenity::{builder::CreateEmbed, http::Http};

use super::OutputTrait;
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
    async fn push(&self, name: &str, entries: &[&Entry]) -> Result<()> {
        let http = Http::new_with_token(&self.token);

        let user = http.get_user(self.user_id).await?;

        for chunk in entries.chunks(10) {
            let embeds: Vec<CreateEmbed> = chunk
                .iter()
                .map(|&entry| {
                    let link = entry.links.first().unwrap();

                    let mut e = CreateEmbed::default();
                    e.title(format!(
                        "{} - {}",
                        name,
                        entry.title.as_ref().unwrap().content
                    ));
                    e.url(link.href.clone());
                    e.timestamp(entry.published.unwrap().to_rfc3339());
                    e
                })
                .collect();

            debug!("pushing {} embeds to discord bot", embeds.len());

            user.dm(&http, |m| {
                m.set_embeds(embeds);
                m
            })
            .await?;
        }

        Ok(())
    }
}
