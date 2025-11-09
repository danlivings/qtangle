use crate::engine::Engine;
use clap::Parser;
use env_logger::Builder;
use log::info;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use qbit_rs::Qbit;
use qbit_rs::model::Credential;
use settings::QTangleSettings;
use std::fmt::Debug;

mod engine;
mod fs;
mod metrics;
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

    if let Some(ref endpoint) = settings.open_telemetry.endpoint {
        info!("Setting up OTLP exporter for collection at {}", endpoint);
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(endpoint.as_ref())
            .build()?;

        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_periodic_exporter(exporter)
            .build();

        opentelemetry::global::set_meter_provider(meter_provider);
    } else {
        info!("OTLP exporter disabled");
    }

    let engine = Engine::new(args.dry_run, api, settings);

    engine.start().await
}
