//! Packs the files to install into one compressed blob that `main.rs`
//! embeds.
//!
//! `SKETCH_SETUP_PAYLOAD` names the staged package folder (the one
//! `tools/package.ps1` builds); `tools/installer.ps1` sets it. Without it
//! the blob is empty, so `cargo build` / `cargo test` of the workspace work
//! and the resulting setup exe just reports that it carries nothing.
//!
//! Blob layout, before deflate: for each file, `u16` name length, the UTF-8
//! name, `u64` byte length, the bytes (little endian). Files only, no
//! subfolders: the package is flat.

use std::io::Write;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=SKETCH_SETUP_PAYLOAD");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("payload.bin");
    let mut raw = Vec::new();
    if let Some(dir) = std::env::var_os("SKETCH_SETUP_PAYLOAD").filter(|d| !d.is_empty()) {
        let dir = PathBuf::from(dir);
        println!("cargo:rerun-if-changed={}", dir.display());
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("SKETCH_SETUP_PAYLOAD {}: {e}", dir.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect();
        files.sort();
        assert!(
            files.iter().any(|f| f.file_name().is_some_and(|n| n == "sketch-app.exe")),
            "SKETCH_SETUP_PAYLOAD has no sketch-app.exe"
        );
        for file in files {
            println!("cargo:rerun-if-changed={}", file.display());
            let name = file.file_name().unwrap().to_string_lossy().into_owned();
            let bytes = std::fs::read(&file).unwrap();
            raw.extend_from_slice(&(name.len() as u16).to_le_bytes());
            raw.extend_from_slice(name.as_bytes());
            raw.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            raw.extend_from_slice(&bytes);
        }
    }
    let mut encoder =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(&raw).unwrap();
    std::fs::write(out, encoder.finish().unwrap()).unwrap();
}
