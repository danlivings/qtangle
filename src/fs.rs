use anyhow::Context;
use std::path::Path;

pub async fn copy_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    let metadata = tokio::fs::metadata(&src).await?;

    if metadata.is_file() {
        let dir = dst.as_ref().parent().context("Could not get parent dir")?;
        tokio::fs::create_dir_all(dir).await?;
        tokio::fs::copy(src, dst).await?;
    } else {
        copy_dir(src, dst).await?;
    }

    Ok(())
}

async fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&dst).await?;
    let mut s = tokio::fs::read_dir(src).await?;
    while let Some(entry) = s.next_entry().await? {
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            Box::pin(copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))).await?;
        } else {
            tokio::fs::copy(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        }
    }
    Ok(())
}
