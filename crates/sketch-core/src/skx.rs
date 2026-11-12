//! .skx dedicated sketch file container — Rust port of `refs/skx-format.js`
//! layered on the minimal ZIP container ported from `refs/zip-container.js`.
//!
//! A `.skx` is a workfile ZIP carrying the sketch model at `state/sketch.json`,
//! a small plaintext summary at `meta/summary.json`, and a `manifest.json`
//! whose `profile` is `"sketch"`. The bytes of the part are the sketch model
//! serialised with `JSON.stringify` and nothing else, so the standalone file
//! and the part embedded in a full `.apx` workfile are byte-identical for the
//! same drawing. Readers identify by content — the mimetype entry and the ZIP
//! magic — never by the extension.

use std::fmt;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/* ── constants ──────────────────────────────────────────────────── */

pub const LOCAL_SIG: u32 = 0x0403_4b50;
pub const CENTRAL_SIG: u32 = 0x0201_4b50;
pub const EOCD_SIG: u32 = 0x0605_4b50;
pub const EOCD_SIZE: usize = 22;
pub const METHOD_STORE: u16 = 0;
pub const METHOD_DEFLATE: u16 = 8;
pub const MIN_VERSION: u16 = 20; // 2.0 (deflate support)
pub const FORMAT: &str = "appraisal-suite.workfile";
pub const FORMAT_VERSION: u32 = 1;

pub const MIME_STRING: &str = "application/vnd.appraisal-suite.sketch";
pub const PROFILE: &str = "sketch";
pub const PART_PATH: &str = "state/sketch.json";
pub const SUMMARY_PATH: &str = "meta/summary.json";
pub const EXTENSION: &str = ".skx";

/* ── errors ─────────────────────────────────────────────────────── */

/// Error raised by the container and the `.skx` envelope. Every corruption
/// path is an error, never a panic: a damaged file must not be silently
/// accepted, and a truncated one must not crash the reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkxError {
    /// The caller asked for something the writer cannot produce.
    Write(String),
    /// The byte stream is damaged or not a container we can read.
    Corrupt(String),
    /// A readable container that does not carry a sketch part.
    NotSketch(String),
    /// A container feature the minimal reader does not implement.
    Unsupported(String),
    /// JSON serialisation/deserialisation failure.
    Json(String),
}

impl fmt::Display for SkxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkxError::Write(m)
            | SkxError::Corrupt(m)
            | SkxError::NotSketch(m)
            | SkxError::Unsupported(m)
            | SkxError::Json(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for SkxError {}

/* ── uint helpers ───────────────────────────────────────────────── */

fn put_u16(v: u16, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_u32(v: u32, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn bad_at(off: usize) -> SkxError {
    SkxError::Corrupt(format!(
        "Corrupt workfile: read past end of data at offset {off}"
    ))
}

/// Bounds-checked little-endian u16 read: truncated input is an error, not a panic.
fn get_u16(bytes: &[u8], off: usize) -> Result<u16, SkxError> {
    let end = off.checked_add(2).ok_or_else(|| bad_at(off))?;
    let slice = bytes.get(off..end).ok_or_else(|| bad_at(off))?;
    Ok(u16::from_le_bytes(slice.try_into().expect("2 bytes")))
}

/// Bounds-checked little-endian u32 read: truncated input is an error, not a panic.
fn get_u32(bytes: &[u8], off: usize) -> Result<u32, SkxError> {
    let end = off.checked_add(4).ok_or_else(|| bad_at(off))?;
    let slice = bytes.get(off..end).ok_or_else(|| bad_at(off))?;
    Ok(u32::from_le_bytes(slice.try_into().expect("4 bytes")))
}

/* ── CRC-32 ─────────────────────────────────────────────────────── */

/// CRC-32 (IEEE, reflected, poly 0xEDB88320) — same digest as the JS table
/// implementation, via the `crc32fast` crate. Strings are hashed as the UTF-8
/// bytes the archive holds, exactly like the JS `crc32(data)` helper.
pub fn crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/* ── compression ────────────────────────────────────────────────── */

/// Raw deflate (ZIP method 8), matching the JS `CompressionStream('deflate-raw')`.
fn deflate_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(bytes, Compression::default());
    let mut out = Vec::new();
    encoder
        .read_to_end(&mut out)
        .expect("deflate into memory cannot fail");
    out
}

fn inflate_bytes(compressed: &[u8]) -> Result<Vec<u8>, SkxError> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| SkxError::Corrupt(format!("Corrupt workfile: deflate stream: {e}")))?;
    Ok(out)
}

/* ── SHA-256 ────────────────────────────────────────────────────── */

/* Port of the JS `sha256Script` fallback: the suite also runs over plain
http where crypto.subtle is undefined, so the reference implementation is
the in-script digest. No extra dependency is added for it. */
const SHA_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 as lowercase hex — the manifest's per-part digests.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let len = bytes.len();
    let total = ((len + 9 + 63) >> 6) << 6;
    let mut buf = vec![0u8; total];
    buf[..len].copy_from_slice(bytes);
    buf[len] = 0x80;
    let bit_len = (len as u64) * 8;
    buf[total - 8..total].copy_from_slice(&bit_len.to_be_bytes());

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    for off in (0..total).step_by(64) {
        let mut w = [0u32; 64];
        for (i, slot) in w.iter_mut().enumerate().take(16) {
            let start = off + i * 4;
            *slot = u32::from_be_bytes(buf[start..start + 4].try_into().expect("4 bytes"));
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let t1 = hh
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ ((!e) & g))
                .wrapping_add(SHA_K[i])
                .wrapping_add(w[i]);
            let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

/* ═════════════════════════════════════════════════════════════════
ZIP WRITER
═════════════════════════════════════════════════════════════════ */

/// One entry handed to [`write_zip`]. Mirrors the JS entry shape
/// `{ path, data, method: 'store'|'deflate' }`; strings become UTF-8 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipEntryInput {
    pub path: String,
    pub data: Vec<u8>,
    /// `METHOD_STORE` or `METHOD_DEFLATE` (the JS default is deflate).
    pub method: u16,
}

impl ZipEntryInput {
    pub fn stored(path: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            data: data.into(),
            method: METHOD_STORE,
        }
    }

    pub fn deflated(path: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            data: data.into(),
            method: METHOD_DEFLATE,
        }
    }
}

struct Prepared {
    path: String,
    method: u16,
    crc: u32,
    uncompressed: usize,
    compressed: Vec<u8>,
}

/// Write a ZIP archive from a slice of entries.
///
/// Mirrors the JS `writeZip`: at least one entry is required, the first entry
/// must be `mimetype`, more than 65535 entries or an archive over 4 GB are
/// rejected (no ZIP64). Flags bit 3 (data descriptor) is not set — sizes are
/// known up front — and mod time/date are left zeroed.
pub fn write_zip(entries: &[ZipEntryInput]) -> Result<Vec<u8>, SkxError> {
    if entries.is_empty() {
        return Err(SkxError::Write(
            "writeZip: at least one entry required (mimetype)".to_owned(),
        ));
    }
    if entries.len() > 65535 {
        return Err(SkxError::Write(
            "ZIP64 not supported: too many entries (>65535)".to_owned(),
        ));
    }
    if entries[0].path != "mimetype" {
        return Err(SkxError::Write(
            "writeZip: first entry must be mimetype".to_owned(),
        ));
    }

    let prepared: Vec<Prepared> = entries
        .iter()
        .map(|e| {
            let method = if e.method == METHOD_STORE {
                METHOD_STORE
            } else {
                METHOD_DEFLATE
            };
            let crc = crc32(&e.data);
            let compressed = if method == METHOD_STORE {
                e.data.clone()
            } else {
                deflate_bytes(&e.data)
            };
            Prepared {
                path: e.path.clone(),
                method,
                crc,
                uncompressed: e.data.len(),
                compressed,
            }
        })
        .collect();

    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    let mut local_offset: u32 = 0;

    for p in &prepared {
        let name = p.path.as_bytes();
        let name_len = name.len();
        let mut header = Vec::with_capacity(30 + name_len);
        put_u32(LOCAL_SIG, &mut header);
        put_u16(MIN_VERSION, &mut header);
        put_u16(0, &mut header); // flags: bit 3 (data descriptor) NOT set; we know sizes
        put_u16(p.method, &mut header);
        put_u16(0, &mut header); // mod time (unused)
        put_u16(0, &mut header); // mod date (unused)
        put_u32(p.crc, &mut header);
        put_u32(p.compressed.len() as u32, &mut header);
        put_u32(p.uncompressed as u32, &mut header);
        put_u16(name_len as u16, &mut header);
        put_u16(0, &mut header); // extra field length
        header.extend_from_slice(name);
        out.extend_from_slice(&header);
        out.extend_from_slice(&p.compressed);

        let mut entry = Vec::with_capacity(46 + name_len);
        put_u32(CENTRAL_SIG, &mut entry);
        put_u16(MIN_VERSION, &mut entry); // version made by
        put_u16(MIN_VERSION, &mut entry); // version needed
        put_u16(0, &mut entry); // flags
        put_u16(p.method, &mut entry);
        put_u16(0, &mut entry); // mod time
        put_u16(0, &mut entry); // mod date
        put_u32(p.crc, &mut entry);
        put_u32(p.compressed.len() as u32, &mut entry);
        put_u32(p.uncompressed as u32, &mut entry);
        put_u16(name_len as u16, &mut entry);
        put_u16(0, &mut entry); // extra field length
        put_u16(0, &mut entry); // file comment length
        put_u16(0, &mut entry); // disk number start
        put_u16(0, &mut entry); // internal attributes
        put_u32(0, &mut entry); // external attributes
        put_u32(local_offset, &mut entry);
        entry.extend_from_slice(name);
        central.extend_from_slice(&entry);

        local_offset = local_offset
            .checked_add((30 + name_len + p.compressed.len()) as u32)
            .ok_or_else(|| {
                SkxError::Write("ZIP64 not supported: archive exceeds 4 GB".to_owned())
            })?;
    }

    let cd_start = out.len();
    out.extend_from_slice(&central);
    let cd_size = out.len() - cd_start;

    let mut eocd = Vec::with_capacity(EOCD_SIZE);
    put_u32(EOCD_SIG, &mut eocd);
    put_u16(0, &mut eocd); // disk number
    put_u16(0, &mut eocd); // disk with CD
    put_u16(prepared.len() as u16, &mut eocd); // entries on this disk
    put_u16(prepared.len() as u16, &mut eocd); // total entries
    put_u32(cd_size as u32, &mut eocd);
    put_u32(cd_start as u32, &mut eocd);
    put_u16(0, &mut eocd); // comment length
    out.extend_from_slice(&eocd);

    if out.len() > u32::MAX as usize {
        return Err(SkxError::Write(
            "ZIP64 not supported: archive exceeds 4 GB".to_owned(),
        ));
    }
    Ok(out)
}

/* ═════════════════════════════════════════════════════════════════
ZIP READER
═════════════════════════════════════════════════════════════════ */

/// Central-directory entry descriptor, mirroring the JS reader's shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipEntry {
    pub path: String,
    pub method: u16,
    pub crc: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub local_offset: u32,
}

/// End-of-central-directory record fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eocd {
    pub disk_num: u16,
    pub cd_disk: u16,
    pub entries_disk: u16,
    pub total_entries: u16,
    pub cd_size: u32,
    pub cd_offset: u32,
    pub comment_len: u16,
}

/// Read the EOCD record from the tail of the blob, searching backward for the
/// signature across the maximum comment length (65535 + 22 bytes).
pub fn read_tail(bytes: &[u8]) -> Result<Eocd, SkxError> {
    let tail_size = bytes.len().min(EOCD_SIZE + 65535);
    let tail = &bytes[bytes.len() - tail_size..];
    if tail.len() < EOCD_SIZE {
        return Err(SkxError::Corrupt(
            "Not a valid ZIP archive (no EOCD signature)".to_owned(),
        ));
    }
    let mut i = tail.len() - EOCD_SIZE;
    loop {
        if tail[i] == 0x50 && tail[i + 1] == 0x4b && tail[i + 2] == 0x05 && tail[i + 3] == 0x06 {
            return Ok(Eocd {
                disk_num: get_u16(tail, i + 4)?,
                cd_disk: get_u16(tail, i + 6)?,
                entries_disk: get_u16(tail, i + 8)?,
                total_entries: get_u16(tail, i + 10)?,
                cd_size: get_u32(tail, i + 12)?,
                cd_offset: get_u32(tail, i + 16)?,
                comment_len: get_u16(tail, i + 20)?,
            });
        }
        if i == 0 {
            return Err(SkxError::Corrupt(
                "Not a valid ZIP archive (no EOCD signature)".to_owned(),
            ));
        }
        i -= 1;
    }
}

/// Parse the central directory and return an entry per file in the archive.
pub fn read_zip_entries(bytes: &[u8]) -> Result<Vec<ZipEntry>, SkxError> {
    let eocd = read_tail(bytes)?;
    let cd_offset = eocd.cd_offset as usize;
    let cd_size = eocd.cd_size as usize;
    let cd_end = cd_offset.checked_add(cd_size).ok_or_else(|| {
        SkxError::Corrupt("Corrupt ZIP: central directory out of range".to_owned())
    })?;
    if cd_end > bytes.len() {
        return Err(SkxError::Corrupt(
            "Corrupt ZIP: central directory out of range".to_owned(),
        ));
    }
    let cd = &bytes[cd_offset..cd_end];

    let mut entries = Vec::with_capacity(eocd.total_entries as usize);
    let mut off = 0usize;
    for _ in 0..eocd.total_entries {
        if get_u32(cd, off)? != CENTRAL_SIG {
            return Err(SkxError::Corrupt(
                "Corrupt ZIP: central directory signature mismatch".to_owned(),
            ));
        }
        let name_len = get_u16(cd, off + 28)? as usize;
        let extra_len = get_u16(cd, off + 30)? as usize;
        let comment_len = get_u16(cd, off + 32)? as usize;
        let name_end = off
            .checked_add(46)
            .and_then(|o| o.checked_add(name_len))
            .ok_or_else(|| bad_at(off))?;
        let name = cd.get(off + 46..name_end).ok_or_else(|| bad_at(off))?;
        entries.push(ZipEntry {
            path: String::from_utf8_lossy(name).into_owned(),
            method: get_u16(cd, off + 10)?,
            crc: get_u32(cd, off + 16)?,
            compressed_size: get_u32(cd, off + 20)?,
            uncompressed_size: get_u32(cd, off + 24)?,
            local_offset: get_u32(cd, off + 42)?,
        });
        off = name_end + extra_len + comment_len;
    }
    Ok(entries)
}

/// Read and decompress a single entry. Reads the fixed 30-byte local header
/// first, then slices exactly the entry's payload — the random access the
/// central directory exists to provide. Verifies size and CRC-32 per entry.
pub fn read_entry(bytes: &[u8], entry: &ZipEntry) -> Result<Vec<u8>, SkxError> {
    let lo = entry.local_offset as usize;
    if bytes.len() < lo + 30 {
        return Err(SkxError::Corrupt(format!(
            "Corrupt workfile: truncated local header at offset {}",
            entry.local_offset
        )));
    }
    if get_u32(bytes, lo)? != LOCAL_SIG {
        return Err(SkxError::Corrupt(format!(
            "Corrupt workfile: local header signature mismatch at offset {}",
            entry.local_offset
        )));
    }
    let name_len = get_u16(bytes, lo + 26)? as usize;
    let extra_len = get_u16(bytes, lo + 28)? as usize;
    let data_start = lo + 30 + name_len + extra_len;
    let data_end = data_start
        .checked_add(entry.compressed_size as usize)
        .ok_or_else(|| bad_at(data_start))?;
    let data = bytes.get(data_start..data_end).ok_or_else(|| {
        SkxError::Corrupt(format!("Corrupt workfile: short read for {}", entry.path))
    })?;

    let plain = match entry.method {
        METHOD_STORE => data.to_vec(),
        METHOD_DEFLATE => inflate_bytes(data)?,
        other => {
            return Err(SkxError::Unsupported(format!(
                "Unsupported ZIP compression method: {other}"
            )))
        }
    };

    if plain.len() != entry.uncompressed_size as usize {
        return Err(SkxError::Corrupt(format!(
            "Corrupt workfile: size mismatch for {}",
            entry.path
        )));
    }
    if crc32(&plain) != entry.crc {
        return Err(SkxError::Corrupt(format!(
            "Corrupt workfile: CRC mismatch for {}",
            entry.path
        )));
    }
    Ok(plain)
}

/// Sniff ZIP magic: `PK\x03\x04`.
pub fn is_zip_bytes(first_bytes: &[u8]) -> bool {
    first_bytes.len() >= 4
        && first_bytes[0] == 0x50
        && first_bytes[1] == 0x4b
        && first_bytes[2] == 0x03
        && first_bytes[3] == 0x04
}

/* ═════════════════════════════════════════════════════════════════
MANIFEST
═════════════════════════════════════════════════════════════════ */

/// Manifest for a workfile container, with the same JSON key order the JS
/// object literal fixes: format, formatVersion, schema, generator, savedAt,
/// assignmentUid, parts, blobs, protection — and `profile` last (the `.skx`
/// writer sets it after `buildManifest`, just like the JS).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub schema: u32,
    pub generator: Generator,
    pub saved_at: String,
    pub assignment_uid: String,
    pub parts: Vec<ManifestPart>,
    pub blobs: Vec<ManifestBlob>,
    pub protection: Protection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generator {
    pub app: String,
    pub build: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestPart {
    pub path: String,
    pub role: String,
    pub bytes: u64,
    pub sha256: String,
    /// `encoding` defaults to `"json"` in the JS builder; kept explicit here.
    pub encoding: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestBlob {
    pub sha256: String,
    pub bytes: u64,
    pub mime: String,
    #[serde(default = "default_blob_refs")]
    pub refs: u64,
}

fn default_blob_refs() -> u64 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Protection {
    pub mode: String,
}

/// Options for [`build_manifest`], mirroring the JS `opts` argument.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ManifestOptions {
    /// Workfile schema version (default 1).
    pub schema: u32,
    pub assignment_uid: String,
    /// ISO string; `None` falls back to "now".
    pub saved_at: Option<String>,
    pub parts: Vec<ManifestPart>,
    pub blobs: Vec<ManifestBlob>,
    /// Defaults to `{mode:'open'}`.
    pub protection: Option<Protection>,
}

/// Build the manifest.json object for a workfile container.
pub fn build_manifest(opts: ManifestOptions) -> Manifest {
    let protection = opts.protection.unwrap_or(Protection {
        mode: "open".to_owned(),
    });
    Manifest {
        format: FORMAT.to_owned(),
        format_version: FORMAT_VERSION,
        schema: if opts.schema == 0 { 1 } else { opts.schema },
        generator: Generator {
            app: "Appraisal Suite".to_owned(),
            build: String::new(),
        },
        saved_at: opts.saved_at.unwrap_or_else(iso8601_now),
        assignment_uid: opts.assignment_uid,
        parts: opts.parts,
        blobs: opts.blobs,
        protection,
        profile: None,
    }
}

/// Parse and validate a manifest: `format` must be the suite workfile format.
pub fn parse_manifest(data: &[u8]) -> Result<Manifest, SkxError> {
    let text = String::from_utf8_lossy(data);
    // The JS parseManifest is JSON.parse + a `format` check only; mirror that
    // order so a manifest with a foreign format reports the format, not a
    // missing-field error, and only then type-check the full shape.
    let value: Value =
        serde_json::from_str(&text).map_err(|e| SkxError::Json(format!("manifest.json: {e}")))?;
    let format = value.get("format").and_then(Value::as_str).unwrap_or("");
    if format != FORMAT {
        return Err(SkxError::Corrupt(format!(
            "Not an Appraisal Suite workfile (format: {format})"
        )));
    }
    serde_json::from_value(value).map_err(|e| SkxError::Json(format!("manifest.json: {e}")))
}

/* ── ISO-8601 timestamp ─────────────────────────────────────────── */

/// Current UTC time as `YYYY-MM-DDTHH:MM:SS.mmmZ` — the shape of the JS
/// `new Date().toISOString()` default for `savedAt`.
pub fn iso8601_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let millis = now.subsec_millis();
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z",
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60
    )
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 to (y, m, d).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/* ═════════════════════════════════════════════════════════════════
.SKX ENVELOPE
═════════════════════════════════════════════════════════════════ */

/// A small, always-plaintext part so a file list or a recovery UI can
/// describe a sketch without inflating and parsing the whole model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub profile: String,
    pub file_name: String,
    pub property_address: String,
    pub pages: usize,
    pub counts: SummaryCounts,
    pub saved_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SummaryCounts {
    pub lines: u64,
    pub polygons: u64,
    pub labels: u64,
    pub symbols: u64,
}

/// Build the `meta/summary.json` content for a model. `subjectInfo` has been
/// both a string and an object across versions; the summary must not throw
/// on either.
pub fn build_summary(model: &Value) -> Summary {
    let pages = model
        .get("pages")
        .and_then(Value::as_array)
        .map(|p| p.as_slice())
        .unwrap_or(&[]);
    let mut counts = SummaryCounts::default();
    for page in pages {
        counts.lines += array_len(page.get("lines"));
        counts.polygons += array_len(page.get("polygons"));
        counts.labels += array_len(page.get("labels"));
        counts.symbols += array_len(page.get("symbols"));
    }

    let address = match model.get("subjectInfo") {
        Some(Value::Object(info)) => info
            .get("propertyAddress")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };

    Summary {
        profile: PROFILE.to_owned(),
        file_name: model
            .get("fileName")
            .and_then(Value::as_str)
            .unwrap_or("Untitled")
            .to_owned(),
        property_address: address,
        pages: pages.len(),
        counts,
        saved_at: model
            .get("savedAt")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(iso8601_now),
    }
}

fn array_len(v: Option<&Value>) -> u64 {
    v.and_then(Value::as_array).map_or(0, |a| a.len() as u64)
}

/// Options for [`write`], mirroring the JS `opts` argument.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkxWriteOptions {
    /// Workfile schema version (default 1).
    pub schema: u32,
    pub assignment_uid: String,
}

/// Result of [`read`]: the model plus whatever manifest/summary survived.
#[derive(Debug, Clone, PartialEq)]
pub struct SkxRead {
    pub model: Value,
    pub manifest: Option<Manifest>,
    pub summary: Option<Value>,
}

/// Result of [`read_any`]: also says whether the input was a ZIP container.
#[derive(Debug, Clone, PartialEq)]
pub struct SkxReadAny {
    pub model: Value,
    pub manifest: Option<Manifest>,
    pub summary: Option<Value>,
    pub container: bool,
}

/// Write a `.skx` from a sketch model (the `buildSaveModel()` output,
/// untouched). Entry order and compression mirror the JS writer: `mimetype`
/// stored, everything else deflated.
pub fn write(model: &Value, opts: &SkxWriteOptions) -> Result<Vec<u8>, SkxError> {
    let has_pages = model.get("pages").is_some_and(Value::is_array);
    if !has_pages {
        return Err(SkxError::Write(
            "SketchFile.write: model has no pages".to_owned(),
        ));
    }

    let json = serde_json::to_string(model).map_err(|e| SkxError::Json(e.to_string()))?;
    let summary =
        serde_json::to_string(&build_summary(model)).map_err(|e| SkxError::Json(e.to_string()))?;
    let part_bytes = json.into_bytes();
    let summary_bytes = summary.into_bytes();

    let saved_at = model
        .get("savedAt")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(iso8601_now);
    let mut manifest = build_manifest(ManifestOptions {
        schema: if opts.schema == 0 { 1 } else { opts.schema },
        assignment_uid: opts.assignment_uid.clone(),
        saved_at: Some(saved_at),
        parts: vec![
            ManifestPart {
                path: SUMMARY_PATH.to_owned(),
                role: "meta.summary".to_owned(),
                bytes: summary_bytes.len() as u64,
                sha256: sha256_hex(&summary_bytes),
                encoding: "json".to_owned(),
            },
            ManifestPart {
                path: PART_PATH.to_owned(),
                role: "app.sketch".to_owned(),
                bytes: part_bytes.len() as u64,
                sha256: sha256_hex(&part_bytes),
                encoding: "json".to_owned(),
            },
        ],
        blobs: Vec::new(),
        protection: Some(Protection {
            mode: "open".to_owned(),
        }),
    });
    manifest.profile = Some(PROFILE.to_owned());

    let manifest_bytes =
        serde_json::to_vec(&manifest).map_err(|e| SkxError::Json(e.to_string()))?;
    write_zip(&[
        ZipEntryInput::stored("mimetype", MIME_STRING.as_bytes().to_vec()),
        ZipEntryInput::deflated("manifest.json", manifest_bytes),
        ZipEntryInput::deflated(SUMMARY_PATH, summary_bytes),
        ZipEntryInput::deflated(PART_PATH, part_bytes),
    ])
}

/// Read a `.skx`, and also any workfile container that happens to carry a
/// `state/sketch.json` part — so a full `.apx` can be opened directly and
/// yield just its sketch.
///
/// A damaged or absent manifest/summary must not cost the appraiser their
/// drawing: the model part carries its own CRC and its own hash, and it is
/// the only part that holds content, so those two parse failures degrade to
/// `None` while read/CRC failures still propagate — the same scope the JS
/// try/catch gives those two parts.
pub fn read(bytes: &[u8]) -> Result<SkxRead, SkxError> {
    let entries = read_zip_entries(bytes)?;
    let find = |path: &str| entries.iter().find(|e| e.path == path);

    let part = find(PART_PATH)
        .ok_or_else(|| SkxError::NotSketch(format!("Not a sketch file: no {PART_PATH} part")))?;
    let model_bytes = read_entry(bytes, part)?;
    let model: Value = serde_json::from_str(&String::from_utf8_lossy(&model_bytes))
        .map_err(|e| SkxError::Json(format!("{}: {e}", PART_PATH)))?;

    let manifest = match find("manifest.json") {
        Some(e) => {
            let bytes_part = read_entry(bytes, e)?;
            parse_manifest(&bytes_part).ok()
        }
        None => None,
    };
    let summary = match find(SUMMARY_PATH) {
        Some(e) => {
            let bytes_part = read_entry(bytes, e)?;
            serde_json::from_str::<Value>(&String::from_utf8_lossy(&bytes_part)).ok()
        }
        None => None,
    };

    Ok(SkxRead {
        model,
        manifest,
        summary,
    })
}

/// Sniff a file without knowing which format it is. ZIP magic selects the
/// container path; anything else is parsed as a legacy `.json` sketch.
pub fn read_any(bytes: &[u8]) -> Result<SkxReadAny, SkxError> {
    if is_zip_bytes(bytes) {
        let result = read(bytes)?;
        Ok(SkxReadAny {
            model: result.model,
            manifest: result.manifest,
            summary: result.summary,
            container: true,
        })
    } else {
        let text = String::from_utf8_lossy(bytes);
        let model: Value = serde_json::from_str(&text)
            .map_err(|e| SkxError::Json(format!("legacy sketch json: {e}")))?;
        Ok(SkxReadAny {
            model,
            manifest: None,
            summary: None,
            container: false,
        })
    }
}

/* ── tests ──────────────────────────────────────────────────────── */

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_model() -> Value {
        json!({
            "fileName": "Test Site",
            "savedAt": "2026-10-21T12:00:00.000Z",
            "subjectInfo": { "propertyAddress": "123 Main St" },
            "pages": [
                {
                    "lines": [
                        { "startPoint": { "x": 0.0, "y": 0.0 }, "endPoint": { "x": 40.0, "y": 0.0 } },
                        { "startPoint": { "x": 40.0, "y": 0.0 }, "endPoint": { "x": 40.0, "y": 28.0 } }
                    ],
                    "polygons": [ { "vertices": [ { "x": 0.0, "y": 0.0 }, { "x": 40.0, "y": 28.0 } ], "closed": true } ],
                    "labels": [ { "textString": "Living", "insertPoint": { "x": 5.0, "y": 5.0 } } ],
                    "symbols": [ { "id": "bath/toilet-res" } ]
                },
                {
                    "lines": [ { "startPoint": { "x": 0.0, "y": 0.0 }, "endPoint": { "x": 1.0, "y": 1.0 } } ],
                    "polygons": [],
                    "labels": [],
                    "symbols": []
                }
            ]
        })
    }

    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // Multi-block input: exercises padding across a full extra block.
        assert_eq!(
            sha256_hex(b"The quick brown fox jumps over the lazy dog"),
            "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
        );
    }

    #[test]
    fn crc32_matches_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn skx_round_trip_is_deep_equal() {
        let model = sample_model();
        let bytes = write(&model, &SkxWriteOptions::default()).expect("write skx");
        let read = read(&bytes).expect("read skx");

        assert_eq!(read.model, model);

        let manifest = read.manifest.expect("manifest present");
        assert_eq!(manifest.format, FORMAT);
        assert_eq!(manifest.format_version, FORMAT_VERSION);
        assert_eq!(manifest.profile.as_deref(), Some(PROFILE));
        assert_eq!(manifest.protection.mode, "open");
        assert_eq!(manifest.saved_at, "2026-10-21T12:00:00.000Z");
        assert_eq!(manifest.parts.len(), 2);
        assert_eq!(manifest.parts[0].path, SUMMARY_PATH);
        assert_eq!(manifest.parts[0].role, "meta.summary");
        assert_eq!(manifest.parts[1].path, PART_PATH);
        assert_eq!(manifest.parts[1].role, "app.sketch");
        // Per-part digests match the parts actually stored.
        for part in &manifest.parts {
            let entry = read_zip_entries(&bytes)
                .expect("entries")
                .into_iter()
                .find(|e| e.path == part.path)
                .expect("part entry");
            let plain = read_entry(&bytes, &entry).expect("part bytes");
            assert_eq!(part.bytes as usize, plain.len());
            assert_eq!(part.sha256, sha256_hex(&plain));
        }

        let summary = read.summary.expect("summary present");
        assert_eq!(
            summary.get("profile").and_then(Value::as_str),
            Some(PROFILE)
        );
        assert_eq!(
            summary.get("fileName").and_then(Value::as_str),
            Some("Test Site")
        );
        assert_eq!(
            summary.get("propertyAddress").and_then(Value::as_str),
            Some("123 Main St")
        );
        assert_eq!(summary.get("pages").and_then(Value::as_u64), Some(2));
        let counts = summary.get("counts").expect("counts");
        assert_eq!(counts.get("lines").and_then(Value::as_u64), Some(3));
        assert_eq!(counts.get("polygons").and_then(Value::as_u64), Some(1));
        assert_eq!(counts.get("labels").and_then(Value::as_u64), Some(1));
        assert_eq!(counts.get("symbols").and_then(Value::as_u64), Some(1));
    }

    #[test]
    fn skx_rewrite_is_byte_identical() {
        let model = sample_model();
        let bytes = write(&model, &SkxWriteOptions::default()).expect("write skx");
        let read = read(&bytes).expect("read skx");
        let again = write(&read.model, &SkxWriteOptions::default()).expect("rewrite skx");
        assert_eq!(bytes, again);
    }

    #[test]
    fn write_rejects_model_without_pages() {
        let err = write(&json!({ "fileName": "x" }), &SkxWriteOptions::default()).unwrap_err();
        assert_eq!(
            err,
            SkxError::Write("SketchFile.write: model has no pages".to_owned())
        );
    }

    #[test]
    fn write_zip_requires_mimetype_first_entry() {
        let err = write_zip(&[ZipEntryInput::deflated("other", b"x".to_vec())]).unwrap_err();
        assert_eq!(
            err,
            SkxError::Write("writeZip: first entry must be mimetype".to_owned())
        );
        let err = write_zip(&[]).unwrap_err();
        assert_eq!(
            err,
            SkxError::Write("writeZip: at least one entry required (mimetype)".to_owned())
        );
    }

    #[test]
    fn read_rejects_missing_sketch_part() {
        let bytes = write_zip(&[
            ZipEntryInput::stored("mimetype", MIME_STRING.as_bytes().to_vec()),
            ZipEntryInput::deflated("other/thing.json", b"{}".to_vec()),
        ])
        .expect("zip");
        let err = read(&bytes).unwrap_err();
        assert_eq!(
            err,
            SkxError::NotSketch("Not a sketch file: no state/sketch.json part".to_owned())
        );
    }

    #[test]
    fn read_rejects_truncated_and_garbage_input_without_panicking() {
        let bytes = write(&sample_model(), &SkxWriteOptions::default()).expect("write skx");

        for len in [
            0usize,
            4,
            21,
            30,
            bytes.len() / 2,
            bytes.len() - 1,
            bytes.len() - 23,
        ] {
            let truncated = &bytes[..len];
            assert!(read(truncated).is_err(), "truncation to {len} must error");
        }

        let garbage: [&[u8]; 3] = [
            b"",
            b"not a zip at all, just text",
            b"PK\x03\x04 truncated after magic",
        ];
        for input in garbage {
            assert!(read(input).is_err(), "garbage input must error");
        }
    }

    #[test]
    fn read_rejects_corrupted_payload_via_crc() {
        let model = sample_model();

        // Deterministic: flip a byte inside the stored mimetype payload
        // (local header is 30 bytes + 8-byte name, so data starts at 38).
        let mut bytes = write(&model, &SkxWriteOptions::default()).expect("write skx");
        let data_start = 30 + "mimetype".len();
        bytes[data_start + 1] ^= 0xff;
        let entries = read_zip_entries(&bytes).expect("central directory still parses");
        let mimetype = entries
            .iter()
            .find(|e| e.path == "mimetype")
            .expect("entry");
        let err = read_entry(&bytes, mimetype).unwrap_err();
        assert!(
            err.to_string().contains("CRC mismatch"),
            "expected CRC mismatch, got: {err}"
        );

        // Flip a byte in the middle of the file; read must error (inflate
        // failure, size mismatch, or CRC mismatch), never panic.
        let mut bytes = write(&model, &SkxWriteOptions::default()).expect("write skx");
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0x01;
        assert!(read(&bytes).is_err());
    }

    #[test]
    fn read_entry_rejects_unsupported_compression_method() {
        let bytes = write_zip(&[ZipEntryInput::stored(
            "mimetype",
            MIME_STRING.as_bytes().to_vec(),
        )])
        .expect("zip");
        let mut entries = read_zip_entries(&bytes).expect("entries");
        entries[0].method = 99;
        let err = read_entry(&bytes, &entries[0]).unwrap_err();
        assert_eq!(
            err,
            SkxError::Unsupported("Unsupported ZIP compression method: 99".to_owned())
        );
    }

    #[test]
    fn parse_manifest_rejects_wrong_format() {
        let err = parse_manifest(br#"{"format":"something.else"}"#).unwrap_err();
        assert_eq!(
            err,
            SkxError::Corrupt(
                "Not an Appraisal Suite workfile (format: something.else)".to_owned()
            )
        );
        assert!(parse_manifest(br#"not json"#).is_err());
    }

    #[test]
    fn read_any_handles_legacy_json_sketch() {
        let model = sample_model();
        let text = serde_json::to_string(&model).expect("json");
        let any = read_any(text.as_bytes()).expect("read_any");
        assert!(!any.container);
        assert_eq!(any.model, model);
        assert!(any.manifest.is_none());
        assert!(any.summary.is_none());

        let bytes = write(&model, &SkxWriteOptions::default()).expect("write skx");
        let any = read_any(&bytes).expect("read_any");
        assert!(any.container);
        assert_eq!(any.model, model);
    }

    #[test]
    fn build_summary_tolerates_both_subject_info_shapes() {
        let string_info = json!({
            "pages": [ { "lines": [ {}, {} ] } ],
            "subjectInfo": "456 Elm St"
        });
        let summary = build_summary(&string_info);
        assert_eq!(summary.property_address, "456 Elm St");
        assert_eq!(summary.file_name, "Untitled");
        assert_eq!(summary.pages, 1);
        assert_eq!(summary.counts.lines, 2);

        let no_pages = json!({ "subjectInfo": { "propertyAddress": "789 Oak" } });
        let summary = build_summary(&no_pages);
        assert_eq!(summary.property_address, "789 Oak");
        assert_eq!(summary.pages, 0);
        assert_eq!(summary.counts, SummaryCounts::default());
    }

    #[test]
    fn civil_date_math_matches_known_instant() {
        // 2026-10-21T12:00:00.000Z is 1792584000 seconds after the epoch.
        let secs: i64 = 1_792_584_000;
        let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
        assert_eq!((y, m, d), (2026, 10, 21));
        let sod = secs.rem_euclid(86_400);
        assert_eq!((sod / 3600, (sod % 3600) / 60, sod % 60), (12, 0, 0));
    }
}
