use brotli::CompressorWriter;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("dist/rr_ui_bg.wasm");
    if !wasm_path.exists() {
        eprintln!("WASM file not found at {:?}", wasm_path);
        std::process::exit(1);
    }

    let mut wasm_file = File::open(wasm_path)?;
    let mut wasm_data = Vec::new();
    wasm_file.read_to_end(&mut wasm_data)?;

    let compressed_path = wasm_path.with_extension("wasm.br");
    let compressed_file = File::create(&compressed_path)?;

    // Level 11 compression as requested
    let mut compressor = CompressorWriter::new(compressed_file, 4096, 11, 22);
    compressor.write_all(&wasm_data)?;
    compressor.flush()?;

    println!("Successfully compressed WASM to {:?}", compressed_path);
    println!("Original size: {} bytes", wasm_data.len());
    println!(
        "Compressed size: {} bytes",
        std::fs::metadata(compressed_path)?.len()
    );

    Ok(())
}
