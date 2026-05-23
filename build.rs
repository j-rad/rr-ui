// build.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch != "wasm32" {
        tonic_prost_build::configure()
            .build_server(true)
            .compile_protos(
                &[
                    "proto/common_protocol.proto",
                    "proto/common_serial.proto",
                    "proto/proxyman.proto",
                    "proto/stats.proto",
                    "proto/rustray.proto",
                    "proto/health.proto",
                ],
                &["."],
            )?;
    }

    // Detect if we are building the server (not wasm)
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch != "wasm32" {
        // Compile Tailwind CSS
        println!("cargo:warning=Compiling Tailwind CSS...");
        let status = std::process::Command::new("npx")
            .args([
                "tailwindcss",
                "-i",
                "./src/input.css",
                "-o",
                "./public/tailwind.css",
                "--minify",
            ])
            .status();

        if let Ok(status) = status {
            if !status.success() {
                println!("cargo:warning=Tailwind CSS compilation failed.");
            }
        } else {
            println!("cargo:warning=npx tailwindcss not found or failed to start.");
        }
    }

    // Add rerun-if-changed for tailwind related files
    println!("cargo:rerun-if-changed=src/input.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=src");

    Ok(())
}
