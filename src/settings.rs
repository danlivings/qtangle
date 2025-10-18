use config::Config;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct QBittorrentSettings<'s> {
    pub username: Cow<'s, str>,
    pub password: Cow<'s, str>,
    pub api_url: Cow<'s, str>,
}

#[derive(Debug, Deserialize)]
pub struct TorrentSettings<'s> {
    #[serde(with = "humantime_serde")]
    pub poll_interval: Duration,
    pub filter_by_tag: Option<Cow<'s, str>>,
}

#[derive(Debug, Deserialize)]
pub struct CopySettings<'s> {
    pub make_hardlinks: bool,
    pub max_concurrency: Option<usize>,
    pub target_folders: HashMap<Cow<'s, str>, Cow<'s, str>>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSettings {
    pub(crate) seed_ratio: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct QTangleSettings<'s> {
    #[serde(borrow)]
    pub qbittorrent: QBittorrentSettings<'s>,
    pub torrent: TorrentSettings<'s>,
    pub copy: CopySettings<'s>,
    pub delete: DeleteSettings,
}

impl QTangleSettings<'_> {
    pub(crate) fn new(config_path: &str) -> anyhow::Result<Self> {
        let settings = Config::builder()
            .add_source(config::File::with_name(config_path).required(false))
            .add_source(
                config::Environment::with_prefix(QTANGLE_ENV_PREFIX)
                    .separator(QTANGLE_ENV_SEPARATOR),
            )
            .build()?;

        let settings = settings.try_deserialize::<QTangleSettings>()?;
        Ok(settings)
    }
}

const QTANGLE_ENV_PREFIX: &str = "QTANGLE";
const QTANGLE_ENV_SEPARATOR: &str = "__";
