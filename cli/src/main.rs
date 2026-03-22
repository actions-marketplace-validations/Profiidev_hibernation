use centaurus::{
  error::{ErrorReportExt, Result},
  init::logging::init_logging,
};
use clap::Parser;

use crate::{cli::Cli, config::Config};

mod api;
mod auth;
mod cli;
mod config;
mod push;

#[tokio::main]
async fn main() -> Result<()> {
  let level = config::log_level();
  init_logging(level);

  let cli = Cli::parse();
  let config = Config::load(cli.config.clone())
    .await
    .context("Failed to load config")?;

  let url = cli
    .url
    .clone()
    .or_else(|| config.as_ref().map(|c| c.app_url.clone()));

  cli.run(config, url).await?;

  Ok(())
}
