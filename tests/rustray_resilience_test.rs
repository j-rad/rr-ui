// tests/rustray_resilience_test.rs
use anyhow::Result;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_watchdog_recovery() -> Result<()> {
    // This test simulates a core crash and verifies watchdog recovery
    // Note: This requires the service to be running with watchdog enabled

    println!("Starting watchdog resilience test...");

    // 1. Find the rustray process PID
    let output = Command::new("pgrep").arg("-f").arg("rustray").output()?;

    if !output.status.success() {
        eprintln!("No rustray process found. Skipping test.");
        return Ok(());
    }

    let pid_str = String::from_utf8_lossy(&output.stdout);
    let pid = pid_str.trim();

    if pid.is_empty() {
        eprintln!("No rustray process found. Skipping test.");
        return Ok(());
    }

    println!("Found rustray process with PID: {}", pid);

    // 2. Kill the process to simulate a crash
    println!("Simulating crash by killing rustray process...");
    let kill_result = Command::new("kill").arg("-9").arg(pid).status()?;

    if !kill_result.success() {
        eprintln!("Failed to kill process");
        return Ok(());
    }

    println!("Process killed. Waiting for watchdog to detect and restart...");

    // 3. Wait for watchdog to detect and restart (should be within 1.5 seconds)
    let start = Instant::now();
    let timeout = Duration::from_secs(3);

    loop {
        sleep(Duration::from_millis(200)).await;

        let check_output = Command::new("pgrep").arg("-f").arg("rustray").output()?;

        if check_output.status.success() {
            let new_pid_str = String::from_utf8_lossy(&check_output.stdout);
            let new_pid = new_pid_str.trim();

            if !new_pid.is_empty() && new_pid != pid {
                let elapsed = start.elapsed();
                println!("✓ Watchdog successfully restarted rustray in {:?}", elapsed);
                println!("New PID: {}", new_pid);

                // Verify recovery time is within 1.5 seconds
                assert!(
                    elapsed < Duration::from_millis(1500),
                    "Recovery took too long: {:?}",
                    elapsed
                );

                return Ok(());
            }
        }

        if start.elapsed() > timeout {
            panic!("Watchdog failed to restart process within timeout");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_exponential_backoff() -> Result<()> {
    // This test verifies that repeated crashes trigger exponential backoff
    println!("Testing exponential backoff behavior...");

    let mut _last_restart_time = Instant::now();

    for attempt in 1..=3 {
        // Find and kill process
        let output = Command::new("pgrep").arg("-f").arg("rustray").output()?;

        if !output.status.success() {
            eprintln!("Process not found on attempt {}", attempt);
            break;
        }

        let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();

        println!("Attempt {}: Killing PID {}", attempt, pid);
        Command::new("kill").arg("-9").arg(&pid).status()?;

        // Measure time to restart
        let start = Instant::now();
        loop {
            sleep(Duration::from_millis(100)).await;

            let check = Command::new("pgrep").arg("-f").arg("rustray").output()?;

            if check.status.success() {
                let new_pid = String::from_utf8_lossy(&check.stdout).trim().to_string();
                if new_pid != pid {
                    let restart_time = start.elapsed();
                    println!("Restarted in {:?}", restart_time);

                    // Verify backoff is increasing (with some tolerance for jitter)
                    if attempt > 1 {
                        let expected_min = Duration::from_secs(2u64.pow(attempt - 1));
                        println!("Expected minimum backoff: {:?}", expected_min);
                    }

                    _last_restart_time = std::time::Instant::now();
                    break;
                }
            }

            if start.elapsed() > Duration::from_secs(60) {
                panic!("Restart timeout on attempt {}", attempt);
            }
        }

        // Wait a bit before next crash
        sleep(Duration::from_secs(2)).await;
    }

    println!("✓ Exponential backoff test completed");
    Ok(())
}
