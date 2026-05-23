//! Embedded Asset Constants
//!
//! All critical CSS assets are baked into the binary at compile time via `include_str!`.
//! This ensures the rr-ui admin panel renders 100% offline with zero CDN dependencies.

/// Pre-compiled Tailwind CSS utility bundle.
pub const TAILWIND_CSS: &str = include_str!("../../../public/tailwind.css");

/// Core layout stylesheet (Obsidian Engine design system).
pub const LAYOUT_CSS: &str = include_str!("../../../public/layout.css");

/// Inter font-face declarations pointing to locally bundled .woff2 files.
pub const INTER_FONT_CSS: &str = include_str!("../../../public/assets/fonts/inter.css");

/// Verifies that all embedded CSS assets are present and non-empty.
/// Logs the verification results. Panics if a critical asset is missing,
/// indicating a corrupt build or broken asset pipeline.
#[cfg(feature = "server")]
pub fn verify_assets() {
    let assets: &[(&str, usize)] = &[
        ("Tailwind CSS", TAILWIND_CSS.len()),
        ("Layout CSS", LAYOUT_CSS.len()),
        ("Inter Font CSS", INTER_FONT_CSS.len()),
    ];

    let mut critical_missing = 0u32;
    let mut total_bytes = 0usize;

    for (name, size) in assets.iter() {
        if *size == 0 {
            log::error!(
                "CRITICAL ASSET MISSING: {} — admin panel will be dehydrated",
                name
            );
            critical_missing += 1;
        } else {
            log::info!("  ✓ rr-ui asset: {} ({} bytes)", name, size);
            total_bytes += size;
        }
    }

    log::info!("rr-ui total embedded CSS payload: {} bytes", total_bytes);

    if critical_missing > 0 {
        panic!(
            "ASSET_DEHYDRATION_ERROR: {} critical asset(s) missing. \
             Cannot serve the admin panel without embedded stylesheets.",
            critical_missing
        );
    }

    log::info!("rr-ui Asset Integrity: 100% OK.");
}
