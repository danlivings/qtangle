use crate::engine::Engine;
use clap::Parser;
use env_logger::Builder;
use qbit_rs::Qbit;
use qbit_rs::model::Credential;
use settings::QTangleSettings;
use std::fmt::Debug;

mod engine;
mod fs;
mod settings;
mod traits;
mod utils;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "qtangle.toml")]
    config_path: String,
    #[arg(short = 'n', long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env("QTANGLE__LOG").init();

    let args = Args::parse();
    let settings = QTangleSettings::new(&*args.config_path)?;

    let credential = Credential::new(
        settings.qbittorrent.username.as_ref(),
        settings.qbittorrent.password.as_ref(),
    );
    let api = Qbit::new(settings.qbittorrent.api_url.as_ref(), credential);

    let engine = Engine::new(args.dry_run, api, settings);

    engine.start().await
}
