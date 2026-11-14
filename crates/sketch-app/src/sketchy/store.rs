//! SQLite container for complete Sketchie revision snapshots (§16).

use std::path::Path;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use serde_json::Value;

use super::{is_format, Result};

const APPLICATION_ID: i64 = 0x534B4359;
const SCHEMA: &str = "
    PRAGMA application_id = 1397441369;
    PRAGMA user_version = 1;
    CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    CREATE TABLE revisions (
        id TEXT PRIMARY KEY, sequence INTEGER NOT NULL, parents TEXT NOT NULL,
        created_at TEXT NOT NULL, author TEXT, message TEXT,
        status TEXT NOT NULL CHECK (status IN ('draft','final')),
        content_hash TEXT NOT NULL, document TEXT NOT NULL
    );";

pub fn is_sqlite(bytes: &[u8]) -> bool {
    bytes.starts_with(b"SQLite format 3\0")
}

fn db<T>(result: rusqlite::Result<T>) -> Result<T> {
    result.map_err(|error| format!("Sketchy store: {error}"))
}

fn connect(path: &Path, writable: bool) -> Result<Connection> {
    let flags = if writable {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    let connection = db(Connection::open_with_flags(
        path,
        flags | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ))?;
    db(connection.pragma_update(None, "trusted_schema", false))?;
    Ok(connection)
}

fn meta(conn: &Connection, key: &str) -> Result<String> {
    db(
        conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
            row.get(0)
        }),
    )
}

fn validate(conn: &Connection) -> Result<String> {
    let app: i64 = db(conn.pragma_query_value(None, "application_id", |row| row.get(0)))?;
    let version: i64 = db(conn.pragma_query_value(None, "user_version", |row| row.get(0)))?;
    if app != APPLICATION_ID || version != 1 {
        return Err("Unsupported SQLite file. Expected Sketchie store version 1.".into());
    }
    let unsafe_schema: bool = db(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type IN ('trigger','view')
            OR (type='table' AND lower(sql) LIKE 'create virtual table%'))",
        [],
        |row| row.get(0),
    ))?;
    if unsafe_schema {
        return Err("Sketchie stores cannot contain triggers, views, or virtual tables.".into());
    }
    let format = meta(conn, "format")?;
    if !is_format(&format) {
        return Err("SQLite file is missing the Sketchie store format marker.".into());
    }
    super::bulge_sign(&meta(conn, "format_version")?)?;
    Ok(format)
}

pub fn read(path: &Path) -> Result<Value> {
    let mut conn = connect(path, false)?;
    let tx = db(conn.transaction())?;
    let format = validate(&tx)?;
    let id = meta(&tx, "head")?;
    let document_id = meta(&tx, "document_id")?;
    let (document, sequence, parents, created_at, author, message, status, hash):
        (String, i64, String, String, Option<String>, Option<String>, String, String) = db(tx.query_row(
        "SELECT document, sequence, parents, created_at, author, message, status, content_hash FROM revisions WHERE id=?1",
        [&id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?))))?;
    let parsed: Value = serde_json::from_str(&document).map_err(|error| error.to_string())?;
    let revision = &parsed["revision"];
    let parents: Value = serde_json::from_str(&parents).map_err(|error| error.to_string())?;
    if parsed["format"] != format
        || parsed["id"] != document_id
        || revision["id"] != id
        || revision["sequence"].as_i64() != Some(sequence)
        || revision["parents"] != parents
        || !parents.is_array()
        || revision["created_at"] != created_at
        || revision["status"] != status
        || revision.pointer("/author/name").and_then(Value::as_str) != author.as_deref()
        || revision["message"].as_str() != message.as_deref()
        || revision["content_hash"] != hash
    {
        return Err("Sketchy store head does not match its revision metadata.".into());
    }
    if content_hash(&parsed)? != hash {
        return Err("Sketchy store head has an invalid content hash.".into());
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sketchy_store_rejects_ordinary_databases_and_unsafe_schema() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/sketchy-tests")
            .join(format!("invalid-store-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("test.sqlite");
        let db = Connection::open(&path).unwrap();
        assert!(read(&path).unwrap_err().contains("Expected Sketchie store"));
        db.execute_batch(SCHEMA).unwrap();
        db.execute_batch("CREATE VIEW unsafe_view AS SELECT key FROM meta")
            .unwrap();
        assert!(read(&path).unwrap_err().contains("cannot contain"));
    }

    #[test]
    fn sketchy_content_hash_ignores_revision_history_and_formats_numbers_canonically() {
        let value = json!({"z":1.0, "a":-0.0, "revision":{"id":"one"}, "history":[]});
        assert_eq!(
            content_hash(&value).unwrap(),
            format!(
                "sha256:{}",
                sketch_core::skx::sha256_hex(br#"{"a":0,"z":1}"#)
            )
        );
    }

    #[test]
    fn sketchie_store_updates_version_metadata_with_the_head() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/sketchy-tests")
            .join(format!("version-metadata-{}", std::process::id()));
        let path = root.join("upgrade.sketchie");
        let document = |version, id, sequence, parents| {
            let mut value = json!({
                "format": super::super::FORMAT, "format_version": version,
                "id": "property", "length_unit": "ft", "structures": [],
                "revision": {"id": id, "sequence": sequence, "parents": parents,
                    "created_at": "2026-11-14T12:00:00Z", "status": "draft"}
            });
            value["revision"]["content_hash"] = json!(content_hash(&value).unwrap());
            value
        };
        let first = document("0.1", "first", 1, json!([]));
        write(&path, &serde_json::to_vec(&first).unwrap(), None).unwrap();
        let second = document("0.2", "second", 2, json!(["first"]));
        let parent = ("first", first["revision"]["content_hash"].as_str().unwrap());
        write(&path, &serde_json::to_vec(&second).unwrap(), Some(parent)).unwrap();
        assert_eq!(read(&path).unwrap(), second);
        let db = connect(&path, true).unwrap();
        assert_eq!(meta(&db, "format_version").unwrap(), "0.2");
        db.execute("UPDATE meta SET value='sketchy' WHERE key='format'", [])
            .unwrap();
        assert!(read(&path).is_err());
    }
}

pub(super) fn content_hash(document: &Value) -> Result<String> {
    let mut content = document
        .as_object()
        .ok_or("Sketchy document must be an object")?
        .clone();
    content.remove("revision");
    content.remove("history");
    let bytes = serde_json_canonicalizer::to_vec(&content).map_err(|error| error.to_string())?;
    Ok(format!("sha256:{}", sketch_core::skx::sha256_hex(&bytes)))
}

pub fn write(path: &Path, bytes: &[u8], previous_head: Option<(&str, &str)>) -> Result<()> {
    let document: Value = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    if let Some(previous_head) = previous_head {
        append(
            &mut connect(path, true)?,
            &document,
            text,
            Some(previous_head),
        )
    } else {
        // Reserve the new path without replacing an existing store or any of
        // its history. An incomplete database is removed if creation fails.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|error| format!("Create Sketchy store: {error}"))?;
        let result =
            connect(path, true).and_then(|mut conn| append(&mut conn, &document, text, None));
        if result.is_err() {
            let _ = std::fs::remove_file(path);
        }
        result
    }
}

fn append(
    conn: &mut Connection,
    document: &Value,
    text: &str,
    previous_head: Option<(&str, &str)>,
) -> Result<()> {
    let revision = &document["revision"];
    let id = revision["id"]
        .as_str()
        .ok_or("Missing Sketchy revision ID")?;
    let sequence = revision["sequence"]
        .as_i64()
        .ok_or("Missing Sketchy revision sequence")?;
    let created_at = revision["created_at"]
        .as_str()
        .ok_or("Missing Sketchy revision timestamp")?;
    let status = revision["status"]
        .as_str()
        .ok_or("Missing Sketchy revision status")?;
    let hash = revision["content_hash"]
        .as_str()
        .ok_or("Missing Sketchy revision content hash")?;
    let document_id = document["id"]
        .as_str()
        .ok_or("Missing Sketchy document ID")?;
    let version = document["format_version"]
        .as_str()
        .ok_or("Missing Sketchy version")?;
    super::bulge_sign(version)?;
    let format = document["format"]
        .as_str()
        .filter(|format| is_format(format))
        .ok_or("Missing or unsupported Sketchie format marker")?;
    let tx = db(conn.transaction_with_behavior(TransactionBehavior::Immediate))?;
    if let Some((previous_head, previous_hash)) = previous_head {
        if validate(&tx)? != format
            || meta(&tx, "head")? != previous_head
            || meta(&tx, "document_id")? != document_id
        {
            return Err("The Sketchy store changed on disk. Reopen it or use Save As.".into());
        }
        let old_hash: Option<String> = db(tx
            .query_row(
                "SELECT content_hash FROM revisions WHERE id=?1",
                [previous_head],
                |row| row.get(0),
            )
            .optional())?;
        if old_hash.as_deref() != Some(previous_hash) {
            return Err("The Sketchy store changed on disk. Reopen it or use Save As.".into());
        }
        if id == previous_head {
            if previous_hash == hash {
                return db(tx.commit());
            }
            return Err("Cannot change an existing Sketchy revision.".into());
        }
    } else {
        db(tx.execute_batch(SCHEMA))?;
        for (key, value) in [("format", format), ("document_id", document_id)] {
            db(tx.execute(
                "INSERT INTO meta(key,value) VALUES (?1,?2)",
                params![key, value],
            ))?;
        }
    }
    db(tx.execute("INSERT INTO revisions (id,sequence,parents,created_at,author,message,status,content_hash,document)
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![id, sequence, revision["parents"].to_string(), created_at,
            revision.pointer("/author/name").and_then(Value::as_str), revision["message"].as_str(), status, hash, text]))?;
    db(tx.execute(
        "INSERT INTO meta(key,value) VALUES ('head',?1), ('format_version',?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![id, version],
    ))?;
    db(tx.commit())
}
