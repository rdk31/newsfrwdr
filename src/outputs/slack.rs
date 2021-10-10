use feed_rs::model::Entry;
use log::debug;
use reqwest::Client;

use serde::Serialize;
use slack_bk::{
    blocks::{Block, Context, ContextElement, Divider, Header, Section},
    composition::{PlainText, Text},
    elements::{Button, Element},
};

use super::OutputTrait;
use crate::Result;
use async_trait::async_trait;

pub struct Slack {
    url: String,
    client: Client,
}

impl Slack {
    pub fn new(url: String, client: Client) -> Self {
        Self { url, client }
    }
}

#[async_trait]
impl OutputTrait for Slack {
    async fn push(&self, name: &str, entries: &[&Entry]) -> Result<()> {
        debug!("pushing {} entries to slack", entries.len());

        for chunk in entries.chunks(10) {
            let blocks: Vec<Block> = chunk
                .iter()
                .map(|&entry| block_from_entry(name, entry))
                .flatten()
                .collect();

            let message = Message { blocks };

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
    blocks: Vec<Block>,
}

fn block_from_entry(name: &str, entry: &Entry) -> [Block; 4] {
    let link = entry.links.first().unwrap();

    let header = Header {
        text: Text::PlainText(PlainText {
            text: format!("{} - {}", name, entry.title.as_ref().unwrap().content),
            emoji: false,
        }),
        block_id: None,
    };

    let section = Section {
        text: Text::PlainText(PlainText {
            text: entry.title.as_ref().unwrap().content.clone(),
            emoji: false,
        })
        .into(),
        accessory: Element::Button(Button {
            text: Text::PlainText(PlainText {
                text: ":link: Open".to_owned(),
                emoji: true,
            }),
            action_id: "button-action".to_owned(),
            url: Some(link.href.clone()),
            ..Default::default()
        })
        .into(),
        ..Default::default()
    };

    let ctx_elements = vec![ContextElement::Text(Text::PlainText(PlainText {
        text: entry
            .published
            .unwrap()
            .format("%d %b %Y %I:%M %p %Z")
            .to_string(),
        emoji: false,
    }))];

    let context = Context {
        elements: ctx_elements,
        ..Default::default()
    };

    [
        Block::Header(header),
        Block::Section(section),
        Block::Context(context),
        Block::Divider(Divider::default()),
    ]
}
