mod config;
mod error;
mod input;
mod outputs;

use std::process;

use futures::future;
use reqwest::Client;

use log::{error, info};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinHandle,
};

use crate::{config::Config, input::Input, outputs::Output};

pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = match Config::from_file("config.toml").await {
        Ok(c) => c,
        Err(e) => {
            error!("Error while reading config: {}", e);
            process::exit(1);
        }
    };

    let client = build_client()?;

    let tasks = watch_inputs(config, client)?;

    future::try_join_all(tasks).await?;

    Ok(())
}

fn build_client() -> Result<Client> {
    let client = Client::builder()
        // .timeout(DEFAULT_TIMEOUT)
        // .user_agent(USER_AGENT)
        .build()?;

    Ok(client)
}

fn watch_inputs(config: Config, client: Client) -> Result<Vec<JoinHandle<Result<()>>>> {
    let mut tasks = Vec::with_capacity(config.inputs.len());
    let (tx, _) = broadcast::channel(tasks.capacity());

    for (name, input_config) in config.inputs.into_iter() {
        info!("Start watcher for \"{}\"", &name);

        let mut outputs = Vec::new();

        // name based outputs
        if let Some(output_configs) = config.outputs.get(&name) {
            for output_config in output_configs {
                outputs.push(Output::new(output_config.clone(), client.clone()));
            }
        }

        // tag based outputs
        for tag in input_config.tags.iter() {
            if let Some(output_configs) = config.outputs.get(tag) {
                for output_config in output_configs {
                    outputs.push(Output::new(output_config.clone(), client.clone()));
                }
            }
        }

        let input = Input::new(name.clone(), input_config, outputs, client.clone());

        let rx = tx.subscribe();

        tasks.push(tokio::spawn(async move {
            if let Err(e) = input.watch(rx).await {
                error!("Watcher for \"{}\" stopped with an error: {}", &name, &e);
                return Err(e);
            }

            info!("Watcher for \"{}\" has stopped", name);

            Ok(())
        }));
    }

    tokio::spawn(async move {
        let mut sig_int = signal(SignalKind::interrupt()).unwrap();
        let mut sig_term = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sig_int.recv() => {},
            _ = sig_term.recv() => {},
        };

        tx.send(()).unwrap();
    });

    Ok(tasks)
}
