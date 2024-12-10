use std::{collections::HashMap, path::Path};

use tokio::io::AsyncBufReadExt;

pub async fn read_file_lines(path: impl AsRef<Path>) -> eyre::Result<Vec<String>> {
    let file = tokio::fs::read(path).await?;
    let mut lines = file.lines();

    let mut contents = vec![];
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            contents.push(trimmed.to_string());
        }
    }

    Ok(contents)
}

pub async fn read_json_to_map(path: impl AsRef<Path>) -> eyre::Result<HashMap<String, String>> {
    let file = tokio::fs::read(path).await?;

    Ok(serde_json::from_slice::<HashMap<String, String>>(&file)?)
}
