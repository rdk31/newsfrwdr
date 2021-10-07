#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("feed rs error: {0}")]
    FeedRs(#[from] feed_rs::parser::ParseFeedError),
    #[error("config parse error: {0}")]
    Config(String),
    #[error("task error: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("serenity error: {0}")]
    Serenity(#[from] serenity::Error),
}
