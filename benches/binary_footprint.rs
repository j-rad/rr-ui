use std::path::PathBuf;
use std::process::Command;

fn main() {
    let debug_path = PathBuf::from("target/debug/rr-ui");
    let release_path = PathBuf::from("target/release/rr-ui");

    if debug_path.exists() {
        let size = std::fs::metadata(&debug_path).unwrap().len();
        println!("Debug Binary Size: {:.2} MB", size as f64 / 1024.0 / 1024.0);
    }

    if release_path.exists() {
        let size = std::fs::metadata(&release_path).unwrap().len();
        println!(
            "Release Binary Size: {:.2} MB",
            size as f64 / 1024.0 / 1024.0
        );
    } else {
        println!("Release binary not found. Run 'cargo build --release' first.");
    }

    // Check for symbol stripping
    if release_path.exists() {
        let output = Command::new("file")
            .arg(&release_path)
            .output()
            .expect("Failed to run 'file' command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains("stripped") {
            println!("Release binary is STRIPPED.");
        } else {
            println!("Release binary is NOT stripped.");
        }
    }
}
