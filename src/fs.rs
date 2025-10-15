use anyhow::Context;
use log::{debug, error};
use std::io::Error;
use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;

pub async fn copy_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    let all_paths = get_all_paths(&src).await?;

    let mut handles = Vec::new();
    for path in all_paths {
        let src = src.as_ref().to_path_buf();
        let dst = dst.as_ref().to_path_buf();
        let handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            let relative_path = path.strip_prefix(&src)?;

            let target_path = if relative_path.as_os_str().is_empty() {
                dst
            } else {
                dst.join(relative_path)
            };

            let target_path_parent = target_path.parent().context("Could not get parent dir")?;
            tokio::fs::create_dir_all(target_path_parent).await?;

            debug!("Copying {:?} to {:?}", relative_path, target_path);

            match tokio::fs::copy(path, &target_path).await {
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
        Some(err) => err,
        None => Ok(()),
    }
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
