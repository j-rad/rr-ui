// src/adapters/atomic_config.rs
//
// Atomic Persistence Layer (APL)
// Provides crash-safe configuration writes using atomic file operations and fsync().

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Atomic configuration writer that prevents corruption via:
/// 1. Write to temporary file
/// 2. fsync() to ensure data is on disk
/// 3. Atomic rename to target path
/// 4. Optional backup of previous version
pub struct AtomicConfigWriter {
    /// Base directory for config files
    base_dir: PathBuf,
    /// Whether to keep .bak files
    keep_backup: bool,
}

impl AtomicConfigWriter {
    /// Create a new atomic config writer
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            keep_backup: true,
        }
    }

    /// Set whether to keep backup files
    pub fn with_backup(mut self, keep_backup: bool) -> Self {
        self.keep_backup = keep_backup;
        self
    }

    /// Write content atomically to the specified path
    ///
    /// # Process
    /// 1. Create temp file in same directory (ensures same filesystem)
    /// 2. Write content to temp file
    /// 3. fsync() temp file
    /// 4. fsync() parent directory (ensures directory entry is persisted)
    /// 5. Rename temp file to target (atomic operation)
    ///
    /// # Errors
    /// Returns error if write, fsync, or rename fails
    pub fn write_atomic(&self, path: &Path, content: &[u8]) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Create temporary file in same directory as target
        let temp_path = self.temp_path(path)?;

        // Write to temp file
        let mut temp_file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;

        temp_file
            .write_all(content)
            .with_context(|| "Failed to write content to temp file")?;

        // Ensure data is written to disk
        temp_file
            .sync_all()
            .with_context(|| "Failed to fsync temp file")?;

        // Close the file explicitly
        drop(temp_file);

        // Sync parent directory to ensure directory entry is persisted
        #[cfg(target_family = "unix")]
        if let Some(parent) = path.parent() {
            if let Ok(dir) = File::open(parent) {
                let _ = dir.sync_all(); // Best effort - don't fail if this doesn't work
            }
        }

        // Atomic rename
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                path.display()
            )
        })?;

        Ok(())
    }

    /// Write with automatic backup of existing file
    ///
    /// If the target file exists, it will be copied to `{path}.bak` before writing.
    pub fn write_with_backup(&self, path: &Path, content: &[u8]) -> Result<()> {
        // Create backup if file exists and backup is enabled
        if self.keep_backup && path.exists() {
            let backup_path = self.backup_path(path);
            fs::copy(path, &backup_path)
                .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;
        }

        self.write_atomic(path, content)
    }

    /// Restore from backup file
    pub fn restore_from_backup(&self, path: &Path) -> Result<()> {
        let backup_path = self.backup_path(path);

        if !backup_path.exists() {
            anyhow::bail!("Backup file does not exist: {}", backup_path.display());
        }

        fs::copy(&backup_path, path)
            .with_context(|| format!("Failed to restore from backup: {}", backup_path.display()))?;

        Ok(())
    }

    /// Validate that a file can be read and parsed as JSON
    pub fn validate_json(&self, path: &Path) -> Result<serde_json::Value> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
    }

    /// Write JSON with validation
    ///
    /// Validates the JSON before writing to ensure it's well-formed.
    pub fn write_json<T: serde::Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        let content =
            serde_json::to_string_pretty(value).with_context(|| "Failed to serialize JSON")?;

        // Validate by parsing back
        let _: serde_json::Value =
            serde_json::from_str(&content).with_context(|| "JSON validation failed")?;

        self.write_with_backup(path, content.as_bytes())
    }

    /// Generate temporary file path in same directory as target
    fn temp_path(&self, path: &Path) -> Result<PathBuf> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let filename = path.file_name().with_context(|| "Invalid file path")?;

        // Use timestamp and process ID for uniqueness
        let temp_name = format!(".{}.tmp.{}", filename.to_string_lossy(), std::process::id());

        Ok(parent.join(temp_name))
    }

    /// Generate backup file path
    fn backup_path(&self, path: &Path) -> PathBuf {
        let mut backup = path.to_path_buf();
        backup.set_extension("bak");
        backup
    }

    /// Clean up temporary files in the base directory
    pub fn cleanup_temp_files(&self) -> Result<usize> {
        let mut count = 0;

        if !self.base_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') && name_str.contains(".tmp.") {
                    fs::remove_file(&path).with_context(|| {
                        format!("Failed to remove temp file: {}", path.display())
                    })?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

/// Transaction log for rollback capability
pub struct TransactionLog {
    log_path: PathBuf,
    entries: Vec<TransactionEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TransactionEntry {
    timestamp: i64,
    operation: String,
    path: PathBuf,
    backup_path: Option<PathBuf>,
}

impl TransactionLog {
    /// Create a new transaction log
    pub fn new(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: log_path.into(),
            entries: Vec::new(),
        }
    }

    /// Load existing transaction log
    pub fn load(log_path: impl Into<PathBuf>) -> Result<Self> {
        let log_path = log_path.into();

        if !log_path.exists() {
            return Ok(Self::new(log_path));
        }

        let content = fs::read_to_string(&log_path)?;
        let entries: Vec<TransactionEntry> = serde_json::from_str(&content)?;

        Ok(Self { log_path, entries })
    }

    /// Record a write operation
    pub fn record_write(&mut self, path: &Path, backup_path: Option<&Path>) -> Result<()> {
        let entry = TransactionEntry {
            timestamp: chrono::Utc::now().timestamp(),
            operation: "write".to_string(),
            path: path.to_path_buf(),
            backup_path: backup_path.map(|p| p.to_path_buf()),
        };

        self.entries.push(entry);
        self.save()
    }

    /// Save transaction log to disk
    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.entries)?;
        let writer =
            AtomicConfigWriter::new(self.log_path.parent().unwrap_or_else(|| Path::new(".")));
        writer.write_atomic(&self.log_path, content.as_bytes())
    }

    /// Rollback the last N transactions
    pub fn rollback(&self, count: usize) -> Result<usize> {
        let mut rolled_back = 0;

        for entry in self.entries.iter().rev().take(count) {
            if let Some(backup_path) = &entry.backup_path {
                if backup_path.exists() {
                    fs::copy(backup_path, &entry.path)
                        .with_context(|| format!("Failed to rollback: {}", entry.path.display()))?;
                    rolled_back += 1;
                }
            }
        }

        Ok(rolled_back)
    }

    /// Clear the transaction log
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_atomic_write() {
        let temp_dir = tempfile::tempdir().unwrap();
        let writer = AtomicConfigWriter::new(temp_dir.path());
        let test_file = temp_dir.path().join("test.json");

        let content = b"{\"test\": \"data\"}";
        writer.write_atomic(&test_file, content).unwrap();

        let mut result = String::new();
        File::open(&test_file)
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();
        assert_eq!(result, String::from_utf8_lossy(content));
    }

    #[test]
    fn test_write_with_backup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let writer = AtomicConfigWriter::new(temp_dir.path());
        let test_file = temp_dir.path().join("test.json");

        // Write initial content
        writer.write_atomic(&test_file, b"original").unwrap();

        // Write with backup
        writer.write_with_backup(&test_file, b"updated").unwrap();

        // Check backup exists
        let backup_path = test_file.with_extension("bak");
        assert!(backup_path.exists());

        let mut backup_content = String::new();
        File::open(&backup_path)
            .unwrap()
            .read_to_string(&mut backup_content)
            .unwrap();
        assert_eq!(backup_content, "original");
    }

    #[test]
    fn test_json_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let writer = AtomicConfigWriter::new(temp_dir.path());
        let test_file = temp_dir.path().join("test.json");

        let data = serde_json::json!({"key": "value"});
        writer.write_json(&test_file, &data).unwrap();

        let validated = writer.validate_json(&test_file).unwrap();
        assert_eq!(validated, data);
    }

    #[test]
    fn test_transaction_log() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("transaction.log");
        let mut log = TransactionLog::new(&log_path);

        let test_path = temp_dir.path().join("test.txt");
        log.record_write(&test_path, None).unwrap();

        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].path, test_path);
    }
}
