use log::{info, LevelFilter};
use std::ffi::CString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use syslog::Facility;

/// System Port for platform detection and integration
pub struct SystemPort;

impl SystemPort {
    /// Detects if the current system is OpenWrt using libc access check
    /// on /etc/openwrt_release
    pub fn is_openwrt() -> bool {
        let path_str =
            std::env::var("RR_OPENWRT_RELEASE").unwrap_or_else(|_| "/etc/openwrt_release".into());
        let c_path = CString::new(path_str).unwrap();
        unsafe { libc::access(c_path.as_ptr(), libc::F_OK) == 0 }
    }
}

/// Initialize logging based on the platform
pub fn init_logging() {
    if SystemPort::is_openwrt() {
        // Setup Syslog
        // Using basic unix syslog connection
        match syslog::init(Facility::LOG_DAEMON, LevelFilter::Info, Some("rr-ui")) {
            Ok(_) => {
                // Syslog initialized
                // We don't use println here as it might write to stdout/stderr which may be discarded or not what we want
            }
            Err(e) => {
                // Fallback to env_logger if syslog fails
                eprintln!("Failed to initialize syslog: {}", e);
                env_logger::init();
            }
        }
        info!("System adapter: OpenWrt detected. Logging redirected to syslog.");
    } else {
        // Default to env_logger on standard linux/others
        env_logger::init();
        info!("System adapter: Standard Linux. Logging to stderr.");
    }
}

/// Helper for atomic writes to storage (flash-safe)
pub struct FlashSafeWriter;

impl FlashSafeWriter {
    /// Writes content to a file atomically by writing to a temp file and renaming it.
    /// This prevents file corruption on power loss during write.
    pub fn write_atomic<P: AsRef<Path>, C: AsRef<[u8]>>(
        path: P,
        content: C,
    ) -> std::io::Result<()> {
        let path = path.as_ref();
        // Create a temp file in the same directory to ensure it's on the same filesystem
        // using the same filename + .tmp extension
        let mut tmp_path_buf = PathBuf::from(path);
        if let Some(filename) = tmp_path_buf.file_name() {
            let mut new_filename = filename.to_os_string();
            new_filename.push(".tmp");
            tmp_path_buf.set_file_name(new_filename);
        } else {
            // Fallback if no filename (shouldn't happen for valid file paths)
            tmp_path_buf = path.with_extension("tmp");
        }

        let tmp_path = tmp_path_buf.as_path();

        {
            let mut file = fs::File::create(tmp_path)?;
            file.write_all(content.as_ref())?;
            // Sync data to storage
            file.sync_all()?;
        }

        // Atomic rename
        fs::rename(tmp_path, path)?;

        // Optionally one could sync the directory, but rename is usually atomic enough for our integrity needs
        Ok(())
    }
}
