//! Browser file picker and download helpers for wasm32 builds.

use rfd::AsyncFileDialog;

/// Prompt the user to pick a single file and return `(file_name, bytes)`.
pub async fn pick_file_bytes(filter: String, extensions: Vec<String>) -> Option<(String, Vec<u8>)> {
    let mut dialog = AsyncFileDialog::new();
    if !extensions.is_empty() {
        let ext_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
        dialog = dialog.add_filter(&filter, &ext_refs);
    }
    let handle = dialog.pick_file().await?;
    let name = handle.file_name();
    let bytes = handle.read().await;
    Some((name, bytes))
}

/// Trigger a browser download with `filename` and `bytes`.
pub async fn download_bytes(filename: String, bytes: Vec<u8>) -> Result<(), String> {
    if let Some(handle) = AsyncFileDialog::new()
        .set_file_name(&filename)
        .save_file()
        .await
    {
        handle.write(&bytes).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}
