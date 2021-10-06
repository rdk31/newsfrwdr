use std::{collections::HashMap, path::Path, time::Duration};

use serde::Deserialize;
use tokio::fs;

use crate::{error::Error, Result};

#[derive(Deserialize)]
pub struct Config {
    pub inputs: HashMap<String, InputConfig>,
    pub outputs: HashMap<String, Vec<OutputConfig>>,
}

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::read(path).await?;
        let config: Config = toml::from_slice(&file[..])?;

        config.is_valid()?;

        Ok(config)
    }

    fn is_valid(&self) -> Result<()> {
        let tags: Vec<&String> = self
            .inputs
            .values()
            .flat_map(|input_config| input_config.tags.iter())
            .collect();

        let names: Vec<&String> = self.inputs.keys().collect();

        for name in names.iter() {
            if tags.contains(name) {
                return Err(Error::Config(format!(
                    "inputs: names and tags are not unique: {}",
                    name
                )));
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct InputConfig {
    pub url: String,

    #[serde(default, with = "humantime_serde")]
    pub interval: Option<Duration>,

    #[serde(default = "default_retry_limit")]
    pub retry_limit: usize,

    #[serde(default = "default_tags")]
    pub tags: Vec<String>,
}

const fn default_retry_limit() -> usize {
    10
}

fn default_tags() -> Vec<String> {
    vec!["default".to_owned()]
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum OutputConfig {
    Discord { url: String },
}
