use crate::fs::{copy_all, hardlink_all};
use crate::metrics;
use crate::settings::QTangleSettings;
use crate::traits::TorrentExt;
use anyhow::Context;
use log::{error, info, warn};
use qbit_rs::Qbit;
use qbit_rs::model::{GetTorrentListArg, Torrent, TorrentFilter};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::Path;

trait EngineCore {
    fn api(&self) -> &Qbit;
    fn settings(&self) -> &QTangleSettings;
    async fn complete(&self, torrent: &Torrent) -> anyhow::Result<()>;
    async fn delete(&self, torrent: &Torrent) -> anyhow::Result<()>;
}

trait EngineExt: EngineCore {
    fn has_copied(&self, torrent: &Torrent) -> bool {
        torrent.tags().contains("qtangle:copied")
    }

    fn has_reached_max_seed_ratio(&self, torrent: &Torrent) -> bool {
        match self.settings().delete.seed_ratio {
            None => false,
            Some(max_seed_ratio) => match torrent.ratio {
                None => false,
                Some(seed_ratio) => max_seed_ratio <= seed_ratio,
            },
        }
    }

    fn guess_tag_and_target_path(&self, torrent: &Torrent) -> Option<(&Cow<str>, &Cow<str>)> {
        let tags = torrent.tags();
        self.settings()
            .copy
            .target_folders
            .iter()
            .find(|(tag, _)| tags.contains(&tag.to_lowercase()))
    }
}

impl<T: EngineCore> EngineExt for T {}

pub struct Engine<'s> {
    inner: EngineImpl<'s>,
}

impl EngineCore for Engine<'_> {
    fn api(&self) -> &Qbit {
        self.inner.api()
    }

    fn settings(&self) -> &QTangleSettings {
        self.inner.settings()
    }

    async fn complete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        self.inner.complete(torrent).await
    }

    async fn delete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        self.inner.delete(torrent).await
    }
}

impl<'s> Engine<'s> {
    pub fn new(dry_run: bool, api: Qbit, settings: QTangleSettings<'s>) -> Self {
        Self {
            inner: EngineImpl::new(dry_run, api, settings),
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(self.settings().torrent.poll_interval);

        loop {
            interval.tick().await;

            match self.process_torrents().await {
                Ok(_) => {}
                Err(err) => error!("{}", err),
            };
        }
    }

    async fn process_torrents(&self) -> anyhow::Result<()> {
        metrics::set_metric_polling_interval(self.settings().torrent.poll_interval);
        if let Some(max_concurrency) = self.settings().copy.max_concurrency {
            metrics::set_metric_max_concurrency(max_concurrency);
        }
        if let Some(seed_ratio) = self.settings().delete.seed_ratio {
            metrics::set_metric_seed_ratio(seed_ratio);
        }

        let mut args = GetTorrentListArg::builder()
            .filter(TorrentFilter::Completed)
            .build();

        args.tag = self
            .settings()
            .torrent
            .filter_by_tag
            .as_ref()
            .map(|s| s.to_string());

        let torrents = self.api().get_torrent_list(args).await?;

        let torrent_counts_by_tag = torrents
            .iter()
            .map(|torrent| {
                self.guess_tag_and_target_path(torrent)
                    .unwrap_or((&Cow::Borrowed("*"), &Cow::Borrowed("")))
                    .0
            })
            .fold(HashMap::new(), |mut acc, tag| {
                acc.entry(tag).and_modify(|count| *count += 1).or_insert(1);
                acc
            });

        for (tag, count) in torrent_counts_by_tag {
            metrics::set_metric_torrents_completed(count, tag);
        }

        for torrent in torrents {
            if !self.has_copied(&torrent) {
                self.complete(&torrent).await?;
            } else if self.has_reached_max_seed_ratio(&torrent) {
                self.delete(&torrent).await?;
            }
        }

        Ok(())
    }
}

enum EngineImpl<'s> {
    DryRun(DryRunEngine<'s>),
    Production(ProductionEngine<'s>),
}

impl<'s> EngineImpl<'s> {
    fn new(dry_run: bool, api: Qbit, settings: QTangleSettings<'s>) -> Self {
        match dry_run {
            true => EngineImpl::DryRun(DryRunEngine { api, settings }),
            false => EngineImpl::Production(ProductionEngine { api, settings }),
        }
    }
}

impl<'s> EngineCore for EngineImpl<'s> {
    fn api(&self) -> &Qbit {
        match self {
            EngineImpl::DryRun(dry_run) => dry_run.api(),
            EngineImpl::Production(production) => production.api(),
        }
    }

    fn settings(&self) -> &QTangleSettings {
        match self {
            EngineImpl::DryRun(dry_run) => dry_run.settings(),
            EngineImpl::Production(production) => production.settings(),
        }
    }

    async fn complete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        match self {
            EngineImpl::DryRun(dry_run) => dry_run.complete(torrent).await,
            EngineImpl::Production(production) => production.complete(torrent).await,
        }
    }

    async fn delete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        match self {
            EngineImpl::DryRun(dry_run) => dry_run.delete(torrent).await,
            EngineImpl::Production(production) => production.delete(torrent).await,
        }
    }
}

struct DryRunEngine<'s> {
    api: Qbit,
    settings: QTangleSettings<'s>,
}

impl<'s> EngineCore for DryRunEngine<'s> {
    fn api(&self) -> &Qbit {
        &self.api
    }

    fn settings(&self) -> &QTangleSettings {
        &self.settings
    }

    async fn complete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        info!("{}", torrent.name());

        if let Some((_, target_path)) = self.guess_tag_and_target_path(torrent) {
            info!("- Copy from {:?} to {}", torrent.save_path, target_path);
        } else if let Some(target_path) = self.settings.copy.target_folders.get("*") {
            info!("- Copy from {:?} to {}", torrent.save_path, target_path);
        } else {
            info!("- Untagged: Nowhere to copy to!");
        }

        Ok(())
    }

    async fn delete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        info!("{}\n- Delete torrent", torrent.name());

        Ok(())
    }
}

struct ProductionEngine<'s> {
    api: Qbit,
    settings: QTangleSettings<'s>,
}

impl<'s> EngineCore for ProductionEngine<'s> {
    fn api(&self) -> &Qbit {
        &self.api
    }

    fn settings(&self) -> &QTangleSettings {
        &self.settings
    }

    async fn complete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        if let Some((tag, target_path)) = self.guess_tag_and_target_path(torrent) {
            self.copy_torrent(torrent, target_path).await?;
            metrics::increment_metric_torrents_processed(tag.as_ref());
        } else if let Some(target_path) = self.settings.copy.target_folders.get("*") {
            self.copy_torrent(torrent, target_path).await?;
            metrics::increment_metric_torrents_processed("*");
        } else {
            warn!(
                "Untagged torrent {} has no catch-all folder to be copied to!",
                torrent.name()
            );
        }

        Ok(())
    }

    async fn delete(&self, torrent: &Torrent) -> anyhow::Result<()> {
        info!("Telling qBittorrent to delete {}", torrent.name());

        let hash = torrent.hash()?;
        self.api.delete_torrents(hash, Some(true)).await?;

        if let Some((tag, _)) = self.guess_tag_and_target_path(torrent) {
            metrics::increment_metric_torrents_processed(tag.as_ref());
        } else {
            metrics::increment_metric_torrents_deleted("*");
        }

        Ok(())
    }
}

impl<'s> ProductionEngine<'s> {
    async fn copy_torrent(
        &self,
        torrent: &Torrent,
        target_path: &Cow<'_, str>,
    ) -> anyhow::Result<()> {
        let content_path = torrent
            .content_path
            .as_ref()
            .with_context(|| format!("{} has no content path", torrent.name()))?;

        let save_path = torrent
            .save_path
            .as_ref()
            .with_context(|| format!("{} has no save path", torrent.name()))?;

        let from = Path::new(&content_path);
        let to = Path::new(target_path.as_ref()).join(from.strip_prefix(&save_path)?);

        info!("Copying {:?} to {:?}", from, to);

        if self.settings.copy.make_hardlinks {
            hardlink_all(&from, &to, self.settings.copy.max_concurrency).await?;
        } else {
            copy_all(&from, &to, self.settings.copy.max_concurrency).await?;
        }

        let hash = torrent.hash()?;
        let new_tags = vec!["qtangle:copied".to_string()];

        self.api.add_torrent_tags(hash, new_tags).await?;
        Ok(())
    }
}
