use anyhow::Context;
use qbit_rs::model::{Hashes, Sep, Torrent};
use std::collections::HashSet;

pub trait TorrentExt {
    fn tags(&self) -> HashSet<String>;
    fn name(&self) -> &str;
    fn hash(&self) -> anyhow::Result<Hashes>;
}

impl TorrentExt for Torrent {
    fn tags(&self) -> HashSet<String> {
        match &self.tags {
            None => HashSet::new(),
            Some(tags) => tags.split(",").map(|s| s.trim().to_lowercase()).collect(),
        }
    }

    fn name(&self) -> &str {
        self.name
            .as_ref()
            .or(self.content_path.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Unknown torrent")
    }

    fn hash(&self) -> anyhow::Result<Hashes> {
        Ok(Hashes::Hashes(Sep::from(vec![
            self.hash
                .as_ref()
                .context("Torrent has no hash")?
                .to_string(),
        ])))
    }
}
