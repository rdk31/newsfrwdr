use std::process::Stdio;

use log::debug;

use tokio::{io::AsyncWriteExt, process::Command};

use super::{Entry, OutputTrait};
use crate::Result;
use async_trait::async_trait;

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
    async fn push(&self, _: &str, entries: &[Entry]) -> Result<()> {
        debug!("pushing {} entries to custom", entries.len());

        for entry in entries {
            let serialized_message = serde_json::to_string(entry).unwrap();

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

                child.wait_with_output().await?;
            } else {
                debug!("pushing to custom: {} ", &self.command);

                let mut args = self.arguments.clone();
                args.push(serialized_message);

                let child = Command::new(&self.command)
                    .args(args)
                    .kill_on_drop(true)
                    .spawn()?;

                child.wait_with_output().await?;
            }
        }

        Ok(())
    }
}
