//! Session persistence manager
//!
//! This module handles saving and loading session data to/from disk.
//! Supports JSONL format (one JSON object per line) for streaming-friendly storage.

use crate::config::types::SessionPersistenceConfig;
use crate::session::types::Session;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Session persistence manager
pub struct SessionPersistenceManager {
    config: SessionPersistenceConfig,
    storage_path: PathBuf,
}

impl SessionPersistenceManager {
    /// Create a new session persistence manager
    pub fn new(config: &SessionPersistenceConfig) -> anyhow::Result<Self> {
        let storage_path = PathBuf::from(&config.storage_path);
        
        // Create storage directory if it doesn't exist
        if !storage_path.exists() {
            create_dir_all(&storage_path)?;
            info!("Created session storage directory: {}", storage_path.display());
        }
        
        Ok(Self {
            config: config.clone(),
            storage_path,
        })
    }
    
    /// Get the file path for a session
    fn get_session_file_path(&self, session_id: &str) -> PathBuf {
        let filename = format!("{}.jsonl", session_id);
        self.storage_path.join(filename)
    }
    
    /// Save a session to disk (JSONL format, one JSON object per line)
    pub async fn save_session(&self, session: &Session) -> anyhow::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let file_path = self.get_session_file_path(&session.id);
        
        // Serialize session to JSON
        let json = serde_json::to_string(session)?;
        
        // Write to file (overwrite)
        let mut file = File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        
        info!("Session saved: {} -> {}", session.id, file_path.display());
        Ok(())
    }
    
    /// Load a session from disk
    pub async fn load_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        let file_path = self.get_session_file_path(session_id);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        // Read file (JSONL format: one JSON object per line)
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);
        
        // Read last line (most recent session state)
        let mut last_line = String::new();
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                last_line = line;
            }
        }
        
        if last_line.is_empty() {
            warn!("Session file is empty: {}", file_path.display());
            return Ok(None);
        }
        
        // Try to deserialize as session::types::Session first
        match serde_json::from_str::<Session>(&last_line) {
            Ok(session) => {
                info!("Session loaded: {} from {}", session_id, file_path.display());
                return Ok(Some(session));
            }
            Err(e) => {
                warn!("Failed to deserialize session {} with new format: {}, trying legacy format", session_id, e);
            }
        }
        
        // Try legacy format (config::types::Session with u64 timestamps)
        // Convert legacy format to new format
        match serde_json::from_str::<serde_json::Value>(&last_line) {
            Ok(mut val) => {
                // Convert u64 timestamps to RFC3339 strings for DateTime<Utc>
                if let Some(created_at) = val.get("created_at").and_then(|v| v.as_u64()) {
                    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at as i64, 0)
                        .unwrap_or_else(chrono::Utc::now);
                    val["created_at"] = serde_json::Value::String(dt.to_rfc3339());
                }
                if let Some(updated_at) = val.get("updated_at").and_then(|v| v.as_u64()) {
                    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at as i64, 0)
                        .unwrap_or_else(chrono::Utc::now);
                    val["updated_at"] = serde_json::Value::String(dt.to_rfc3339());
                }
                
                match serde_json::from_value::<Session>(val) {
                    Ok(session) => {
                        info!("Session loaded (legacy format converted): {} from {}", session_id, file_path.display());
                        return Ok(Some(session));
                    }
                    Err(e) => {
                        warn!("Failed to convert legacy session {}: {}", session_id, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse session file {}: {}", session_id, e);
            }
        }
        
        Ok(None)
    }
    
    /// Delete a session from disk
    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let file_path = self.get_session_file_path(session_id);
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
            info!("Session deleted: {} from {}", session_id, file_path.display());
        } else {
            warn!("Session file not found: {}", file_path.display());
        }
        
        Ok(())
    }
    
    /// List all persisted sessions
    pub async fn list_sessions(&self) -> anyhow::Result<Vec<String>> {
        if !self.config.enabled {
            return Ok(vec![]);
        }
        
        let mut session_ids = vec![];
        
        // Read all .jsonl files in storage directory
        let entries = std::fs::read_dir(&self.storage_path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                if let Some(filename) = path.file_stem() {
                    if let Some(session_id) = filename.to_str() {
                        session_ids.push(session_id.to_string());
                    }
                }
            }
        }
        
        info!("Found {} persisted sessions in {}", session_ids.len(), self.storage_path.display());
        Ok(session_ids)
    }
    
    /// Check if persistence is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Get storage path
    pub fn get_storage_path(&self) -> &Path {
        &self.storage_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::SessionPersistenceConfig;
    
    #[test]
    fn test_persistence_config() {
        let config = SessionPersistenceConfig {
            enabled: true,
            storage_path: "./data/sessions".to_string(),
            auto_save_interval: 300,
            format: "jsonl".to_string(),
        };
        
        assert_eq!(config.enabled, true);
        assert_eq!(config.storage_path, "./data/sessions");
        assert_eq!(config.auto_save_interval, 300);
        assert_eq!(config.format, "jsonl");
    }
}
