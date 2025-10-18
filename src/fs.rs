use crate::utils::chunk;
use anyhow::Context;
use log::{debug, error};
use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;

pub async fn hardlink_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    max_concurrency: Option<usize>,
) -> anyhow::Result<()> {
    process_all(src, dst, max_concurrency, CopyMode::Hardlink).await
}

pub async fn copy_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    max_concurrency: Option<usize>,
) -> anyhow::Result<()> {
    process_all(src, dst, max_concurrency, CopyMode::Copy).await
}

#[derive(Clone)]
enum CopyMode {
    Hardlink,
    Copy,
}

async fn process_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    max_concurrency: Option<usize>,
    mode: CopyMode,
) -> anyhow::Result<()> {
    let all_paths = get_all_paths(&src).await?;

    match max_concurrency {
        Some(max_concurrency) => debug!("Using chunk size of {:}", max_concurrency),
        None => debug!("No chunk size, all tasks will be processed concurrently"),
    }

    for chunk in chunk(all_paths, max_concurrency) {
        debug!("Starting task");

        let mut handles = Vec::new();
        for path in chunk {
            let src = src.as_ref().to_path_buf();
            let dst = dst.as_ref().to_path_buf();
            let mode = mode.clone();
            let handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
                let relative_path = path.strip_prefix(&src)?;

                let target_path = if relative_path.as_os_str().is_empty() {
                    dst
                } else {
                    dst.join(relative_path)
                };

                let target_path_parent =
                    target_path.parent().context("Could not get parent dir")?;
                tokio::fs::create_dir_all(target_path_parent).await?;

                let result = match mode {
                    CopyMode::Hardlink => {
                        debug!("Creating hardlink from {:?} to {:?}", path, target_path);
                        tokio::fs::hard_link(&path, &target_path).await
                    }
                    CopyMode::Copy => {
                        debug!("Copying {:?} to {:?}", path, target_path);
                        tokio::fs::copy(&path, &target_path).await.map(|_| ())
                    }
                };

                match result {
                    Ok(_) => {}
                    Err(err) => {
                        error!("{}: {}", target_path.display(), err);
                        Err(err)?
                    }
                };

                Ok(())
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await?);
        }

        match results.into_iter().find(|result| result.is_err()) {
            Some(res) => res?,
            None => {}
        }

        debug!("Task finished");
    }

    Ok(())
}

async fn get_all_paths(path: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let metadata = path.as_ref().metadata()?;

    let mut result = Vec::new();

    if metadata.is_file() {
        result.push(path.as_ref().to_path_buf());
    } else if metadata.is_dir() {
        let mut read_dir = tokio::fs::read_dir(path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            let mut sub_paths = Box::pin(get_all_paths(path)).await?;
            result.append(&mut sub_paths);
        }
    }

    Ok(result)
}
