use std::process::Stdio;

use chrono::{DateTime, Utc};
use log::debug;
use serde::Serialize;
use tokio::{io::AsyncWriteExt, process::Command};

use super::OutputTrait;
use crate::Result;
use async_trait::async_trait;
use feed_rs::model::Entry;

pub struct Custom {
    command: String,
    arguments: Vec<String>,
    use_stdin: bool,
}

impl Custom {
    pub fn new(command: String, arguments: Vec<String>, use_stdin: bool) -> Self {
        Self {
            command,
            arguments,
            use_stdin,
        }
    }
}

#[async_trait]
impl OutputTrait for Custom {
    async fn push(&self, name: &str, entries: &[&Entry]) -> Result<()> {
        let entries = entries
            .iter()
            .map(|&entry| EntryObject::new(name, entry.clone()))
            .collect();

        let message = Message { entries };
        let serialized_message = serde_json::to_string(&message).unwrap();

        if self.use_stdin {
            debug!("pushing to custom stdin: {}", &self.command);

            let mut child = Command::new(&self.command)
                .args(&self.arguments)
                .stdin(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;

            let child_stdin = child.stdin.as_mut().unwrap();
            child_stdin.write_all(serialized_message.as_bytes()).await?;
            child_stdin.flush().await?;
            drop(child_stdin);

            child.wait_with_output().await?;
        } else {
            debug!("pushing to custom: {} ", &self.command);

            println!("{:?}", &serialized_message);

            let mut args = self.arguments.clone();
            args.push(serialized_message);

            let child = Command::new(&self.command)
                .args(args)
                .kill_on_drop(true)
                .spawn()?;

            child.wait_with_output().await?;
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct Message {
    entries: Vec<EntryObject>,
}

#[derive(Serialize)]
struct EntryObject {
    title: String,
    url: String,
    timestamp: DateTime<Utc>,
}

impl EntryObject {
    fn new(name: &str, entry: Entry) -> Self {
        let link = entry.links.first().unwrap();

        Self {
            title: format!("{} - {}", name, entry.title.unwrap().content),
            url: link.href.clone(),
            timestamp: entry.published.unwrap(),
        }
    }
}
