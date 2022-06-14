use log::debug;
use reqwest::Client;

use serde::Serialize;
use slack_bk::{
    blocks::{Block, Context, ContextElement, Divider, Header, Section},
    composition::{MarkdownText, PlainText, Text},
    elements::{Button, Element},
};

use super::{Entry, OutputTrait};
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
    async fn push(&self, _: &str, entries: &[Entry]) -> Result<()> {
        debug!("pushing {} entries to slack", entries.len());

        for chunk in entries.chunks(10) {
            let blocks: Vec<Block> = chunk.iter().flat_map(block_from_entry).collect();

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

fn block_from_entry(entry: &Entry) -> [Block; 4] {
    let header = Header {
        text: Text::PlainText(PlainText {
            text: entry.title.clone(),
            emoji: false,
        }),
        block_id: None,
    };

    let description = if entry.description.is_empty() {
        "no description".to_owned()
    } else {
        entry.description.clone()
    };

    let section = Section {
        text: Text::Markdown(MarkdownText {
            text: description,
            verbatim: false,
        })
        .into(),
        accessory: Element::Button(Button {
            text: Text::PlainText(PlainText {
                text: ":link: Open".to_owned(),
                emoji: true,
            }),
            action_id: "button-action".to_owned(),
            url: Some(entry.url.clone()),
            ..Default::default()
        })
        .into(),
        ..Default::default()
    };

    let mut ctx_elements = Vec::with_capacity(3);

    if let Some(author) = entry.author.as_ref() {
        ctx_elements.push(ContextElement::Text(Text::Markdown(MarkdownText {
            text: author.clone(),
            verbatim: false,
        })));
    }

    ctx_elements.push(ContextElement::Text(Text::Markdown(MarkdownText {
        text: entry.url.clone(),
        verbatim: false,
    })));

    ctx_elements.push(ContextElement::Text(Text::PlainText(PlainText {
        text: entry.timestamp.format("%d %b %Y %I:%M %p %Z").to_string(),
        emoji: false,
    })));

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
