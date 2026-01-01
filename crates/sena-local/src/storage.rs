use async_trait::async_trait;
use chrono::Utc;
use sena_core::{Error, KeyValueStore, Result};
use std::path::Path;
use std::sync::Arc;
use tokio_rusqlite::Connection;

pub struct SqliteStore {
    conn: Arc<Connection>,
}

impl SqliteStore {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .await
            .map_err(|e| Error::internal(format!("Failed to open database: {}", e)))?;

        conn.call(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS kv_store (
                    key TEXT PRIMARY KEY,
                    value BLOB NOT NULL,
                    expires_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_kv_expires ON kv_store(expires_at);
                "#,
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to initialize database: {}", e)))?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn in_memory() -> Result<Self> {
        let conn = Connection::open(":memory:")
            .await
            .map_err(|e| Error::internal(format!("Failed to open in-memory database: {}", e)))?;

        conn.call(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS kv_store (
                    key TEXT PRIMARY KEY,
                    value BLOB NOT NULL,
                    expires_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_kv_expires ON kv_store(expires_at);
                "#,
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to initialize database: {}", e)))?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn cleanup_expired(&self) -> Result<usize> {
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let deleted = conn.execute(
                "DELETE FROM kv_store WHERE expires_at IS NOT NULL AND expires_at < ?1",
                [now],
            )?;
            Ok(deleted)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to cleanup expired entries: {}", e)))
    }
}

#[async_trait]
impl KeyValueStore for SqliteStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let key = key.to_string();
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT value FROM kv_store WHERE key = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
            )?;
            let result: Option<Vec<u8>> = stmt
                .query_row([&key, &now.to_string()], |row| row.get(0))
                .ok();
            Ok(result)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to get key: {}", e)))
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let key = key.to_string();
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT INTO kv_store (key, value, expires_at, created_at, updated_at)
                VALUES (?1, ?2, NULL, ?3, ?3)
                ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3
                "#,
                rusqlite::params![key, value, now],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to set key: {}", e)))
    }

    async fn set_with_ttl(&self, key: &str, value: Vec<u8>, ttl_secs: u64) -> Result<()> {
        let key = key.to_string();
        let now = Utc::now().timestamp();
        let expires_at = now + ttl_secs as i64;
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT INTO kv_store (key, value, expires_at, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?4)
                ON CONFLICT(key) DO UPDATE SET value = ?2, expires_at = ?3, updated_at = ?4
                "#,
                rusqlite::params![key, value, expires_at, now],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to set key with TTL: {}", e)))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let key = key.to_string();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            conn.execute("DELETE FROM kv_store WHERE key = ?1", [&key])?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to delete key: {}", e)))
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let key = key.to_string();
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM kv_store WHERE key = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
                    [&key, &now.to_string()],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            Ok(exists)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to check key existence: {}", e)))
    }

    async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let pattern = pattern.replace('*', "%").replace('?', "_");
        let now = Utc::now().timestamp();
        let conn = self.conn.clone();

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT key FROM kv_store WHERE key LIKE ?1 AND (expires_at IS NULL OR expires_at > ?2)",
            )?;
            let keys: Vec<String> = stmt
                .query_map([&pattern, &now.to_string()], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(keys)
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to list keys: {}", e)))
    }

    async fn clear(&self) -> Result<()> {
        let conn = self.conn.clone();

        conn.call(|conn| {
            conn.execute("DELETE FROM kv_store", [])?;
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to clear store: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_store_basic() {
        let store = SqliteStore::in_memory().await.unwrap();

        store.set("key1", b"value1".to_vec()).await.unwrap();
        let value = store.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        assert!(store.exists("key1").await.unwrap());
        assert!(!store.exists("nonexistent").await.unwrap());

        store.delete("key1").await.unwrap();
        assert!(!store.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_store_ttl() {
        let store = SqliteStore::in_memory().await.unwrap();

        store
            .set_with_ttl("expiring", b"data".to_vec(), 3600)
            .await
            .unwrap();
        assert!(store.exists("expiring").await.unwrap());

        store
            .set_with_ttl("expired", b"data".to_vec(), 0)
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(!store.exists("expired").await.unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_store_keys_pattern() {
        let store = SqliteStore::in_memory().await.unwrap();

        store.set("session:1", b"s1".to_vec()).await.unwrap();
        store.set("session:2", b"s2".to_vec()).await.unwrap();
        store.set("user:1", b"u1".to_vec()).await.unwrap();

        let session_keys = store.keys("session:*").await.unwrap();
        assert_eq!(session_keys.len(), 2);

        let all_keys = store.keys("*").await.unwrap();
        assert_eq!(all_keys.len(), 3);
    }
}
