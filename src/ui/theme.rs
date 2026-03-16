//! Obsidian Engine Design System
//!
//! A high-tech "Command Center" aesthetic for EdgeRay.
//! Uses Deep Obsidian (#020203) as Level 0, with data surfaces on Level 1 (Glass)
//! and technical overlays on Level 2 (HUD).
//!
//! # Palette
//! - Void: #020203
//! - Cyber Purple: #BC13FE
//! - Electric Cyan: #00F2FF
//! - Emerald: #10B981
//!
//! # Surfaces
//! - Level 0 (Void): Deep obsidian background
//! - Level 1 (Glass): Backdrop-blur panels with subtle borders
//! - Level 2 (HUD): Technical overlays and highlight elements

// ─── Color Constants ───────────────────────────────────────────────────────────

/// Deep obsidian void — the absolute base layer (#020203)
pub const COLOR_VOID: &str = "#020203";

/// Cyber purple accent — primary action color (#BC13FE)
pub const COLOR_CYBER_PURPLE: &str = "#BC13FE";

/// Electric cyan — secondary/connected state (#00F2FF)
pub const COLOR_ELECTRIC_CYAN: &str = "#00F2FF";

/// Emerald — success state (#10B981)
pub const COLOR_EMERALD: &str = "#10B981";

/// Amber — warning/DPI interference (#F59E0B)
pub const COLOR_AMBER: &str = "#F59E0B";

/// Error red (#EF4444)
pub const COLOR_ERROR: &str = "#EF4444";

// ─── Tailwind Class Strings ────────────────────────────────────────────────────

/// Level 0 — full-screen void background
pub const VOID_BG: &str = "bg-[#020203] min-h-screen text-white antialiased";

/// Level 1 — frosted glass surface
/// Backdrop-blur-xl with white/5 background and white/10 border
pub const GLASS_SURFACE: &str =
    "bg-white/[0.05] backdrop-blur-xl border border-white/[0.10] rounded-2xl shadow-2xl";

/// Level 1 — glass surface with hover interaction
pub const GLASS_SURFACE_HOVER: &str = "bg-white/[0.05] backdrop-blur-xl border border-white/[0.10] rounded-2xl shadow-2xl hover:border-[#BC13FE]/30 hover:shadow-[0_0_30px_rgba(188,19,254,0.08)] transition-all duration-300";

/// Level 2 — HUD overlay elements (higher z, sharper border)
pub const HUD_OVERLAY: &str =
    "bg-white/[0.08] backdrop-blur-2xl border border-white/[0.15] rounded-xl shadow-lg";

/// Glass panel padding
pub const GLASS_PADDING: &str = "p-5";

/// Glass panel inner glow gradient overlay (pointer-events-none)
pub const GLASS_INNER_GLOW: &str = "absolute inset-0 bg-gradient-to-br from-white/[0.03] via-transparent to-black/[0.02] pointer-events-none rounded-2xl";

// ─── Typography ────────────────────────────────────────────────────────────────

/// General UI text — Inter font
pub const FONT_GENERAL: &str = "font-['Inter',ui-sans-serif,system-ui,sans-serif]";

/// Telemetry / numeric data — JetBrains Mono for technical alignment
pub const FONT_TELEMETRY: &str = "font-['JetBrains_Mono',ui-monospace,monospace]";

/// Large heading style
pub const HEADING_LG: &str = "text-xl font-bold tracking-tight text-white";

/// Medium heading style
pub const HEADING_MD: &str = "text-lg font-semibold tracking-tight text-white";

/// Small heading style
pub const HEADING_SM: &str = "text-sm font-semibold tracking-tight text-white/90";

/// Body text
pub const TEXT_BODY: &str = "text-sm text-white/70 leading-relaxed";

/// Secondary text
pub const TEXT_SECONDARY: &str = "text-xs text-white/50";

/// Telemetry value — large mono number
pub const TELEMETRY_VALUE: &str =
    "font-['JetBrains_Mono',ui-monospace,monospace] text-2xl font-bold tabular-nums tracking-tight";

/// Telemetry label — small secondary
pub const TELEMETRY_LABEL: &str = "text-xs text-white/50 uppercase tracking-widest";

// ─── Border & Glow Effects ─────────────────────────────────────────────────────

/// Specular highlight glow for glass edges (cyber purple)
pub const GLOW_PURPLE: &str = "shadow-[0_0_20px_rgba(188,19,254,0.15)]";

/// Connected state glow (electric cyan)
pub const GLOW_CYAN: &str = "shadow-[0_0_20px_rgba(0,242,255,0.15)]";

/// Success glow (emerald)
pub const GLOW_EMERALD: &str = "shadow-[0_0_20px_rgba(16,185,129,0.15)]";

/// Warning/DPI interference glow (amber)
pub const GLOW_AMBER: &str = "shadow-[0_0_20px_rgba(245,158,11,0.15)]";

/// Specular border — active edge highlight
pub const BORDER_SPECULAR: &str = "border-[#BC13FE]/40";

/// Default border — subtle glass edge
pub const BORDER_GLASS: &str = "border-white/[0.10]";

// ─── Forensics / Handshake Tracer ──────────────────────────────────────────────

/// Radius (px) of a timeline pulse node at rest
pub const TIMELINE_NODE_RADIUS: f32 = 10.0;

/// Maximum expansion radius (px) when a pulse "ignites"
pub const TIMELINE_PULSE_RADIUS: f32 = 22.0;

/// Width of the vertical timeline spine
pub const TIMELINE_SPINE_WIDTH: f32 = 2.0;

/// Vertical spacing between timeline nodes (px)
pub const TIMELINE_NODE_SPACING: f32 = 80.0;

/// Routing-canvas node radius (px)
pub const ROUTING_NODE_RADIUS: f32 = 24.0;

/// Routing-canvas animated packet-dot radius (px)
pub const ROUTING_DOT_RADIUS: f32 = 4.0;

/// Routing link default stroke color (white/20)
pub const ROUTING_LINK_COLOR: &str = "rgba(255,255,255,0.20)";

/// Routing link active stroke color (cyan/40)
pub const ROUTING_LINK_ACTIVE: &str = "rgba(0,242,255,0.40)";

// ─── Scanline Texture ──────────────────────────────────────────────────────────

/// CRT scanline SVG pattern (inline, 2.5% opacity)
pub const SCANLINE_OPACITY: f32 = 0.025;

/// Scanline line spacing in pixels
pub const SCANLINE_SPACING: u32 = 4;

/// Scanline line thickness in pixels
pub const SCANLINE_THICKNESS: f32 = 1.0;

// ─── Grid Texture ──────────────────────────────────────────────────────────────

/// Technical grid line spacing (2px grid)
pub const GRID_SIZE: u32 = 2;

/// Grid line opacity
pub const GRID_OPACITY: f32 = 0.03;

// ─── Animation Constants ───────────────────────────────────────────────────────

/// Default transition for all interactive elements
pub const TRANSITION_DEFAULT: &str = "transition-all duration-300 ease-out";

/// Fast transition for micro-interactions
pub const TRANSITION_FAST: &str = "transition-all duration-150 ease-out";

/// Slow transition for state changes
pub const TRANSITION_SLOW: &str = "transition-all duration-500 ease-in-out";

/// GPU-accelerated transform hint
pub const GPU_ACCELERATED: &str = "will-change-transform";

// ─── Specular Highlight Calculation ────────────────────────────────────────────

/// Distance threshold in pixels below which border edges "ignite" with a specular glow.
pub const SPECULAR_THRESHOLD_PX: f64 = 50.0;

/// Calculates the fractional intensity of a specular highlight based on the
/// pointer position relative to the element bounds.
///
/// Returns a value in `[0.0, 1.0]` where 1.0 means the pointer is exactly on
/// the nearest border, and 0.0 means it is at or beyond `SPECULAR_THRESHOLD_PX`.
///
/// # Arguments
/// * `pointer_x` — horizontal cursor position relative to the element's left edge.
/// * `pointer_y` — vertical cursor position relative to the element's top edge.
/// * `width` — element width in pixels.
/// * `height` — element height in pixels.
///
/// # Returns
/// `(intensity, nearest_edge)` where `nearest_edge` encodes which border is
/// closest: 0 = top, 1 = right, 2 = bottom, 3 = left.
pub fn specular_intensity(pointer_x: f64, pointer_y: f64, width: f64, height: f64) -> (f64, u8) {
    // Distance to each of the four edges
    let dist_top = pointer_y;
    let dist_right = width - pointer_x;
    let dist_bottom = height - pointer_y;
    let dist_left = pointer_x;

    // Find the minimum distance and which edge it belongs to
    let distances = [
        (dist_top, 0u8),
        (dist_right, 1u8),
        (dist_bottom, 2u8),
        (dist_left, 3u8),
    ];

    let (min_dist, nearest_edge) = distances
        .iter()
        .copied()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((f64::MAX, 0));

    // If outside the element entirely or beyond threshold, no glow
    if min_dist < 0.0 || min_dist >= SPECULAR_THRESHOLD_PX {
        return (0.0, nearest_edge);
    }

    // Linear falloff: intensity = 1.0 at border, 0.0 at threshold
    let intensity = 1.0 - (min_dist / SPECULAR_THRESHOLD_PX);
    (intensity.clamp(0.0, 1.0), nearest_edge)
}

/// Generates an inline CSS `box-shadow` string for the specular highlight.
///
/// The shadow is a cyber-purple glow positioned on the nearest edge, with
/// spread and opacity proportional to `intensity`.
pub fn specular_box_shadow(intensity: f64, nearest_edge: u8) -> String {
    if intensity <= 0.0 {
        return String::new();
    }

    let alpha = (intensity * 0.4).min(0.4);
    let spread = (intensity * 12.0).round() as i32;

    // Offset the glow toward the nearest edge
    let (offset_x, offset_y) = match nearest_edge {
        0 => (0, -(spread / 2)), // top
        1 => (spread / 2, 0),    // right
        2 => (0, spread / 2),    // bottom
        3 => (-(spread / 2), 0), // left
        _ => (0, 0),
    };

    format!(
        "{}px {}px {}px rgba(188, 19, 254, {:.3})",
        offset_x, offset_y, spread, alpha
    )
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specular_at_center_is_zero() {
        // The center of a 200×100 element is 50px from the nearest border (top/bottom),
        // which is exactly at the threshold, so intensity should be 0.0.
        let (intensity, _edge) = specular_intensity(100.0, 50.0, 200.0, 100.0);
        assert!(
            intensity.abs() < f64::EPSILON,
            "Center of 200×100 should have zero intensity, got {intensity}"
        );
    }

    #[test]
    fn test_specular_at_border_is_one() {
        // Pointer on the left edge (x=0)
        let (intensity, edge) = specular_intensity(0.0, 50.0, 200.0, 100.0);
        assert!(
            (intensity - 1.0).abs() < f64::EPSILON,
            "Left border should have intensity 1.0, got {intensity}"
        );
        assert_eq!(edge, 3, "Nearest edge should be left (3)");
    }

    #[test]
    fn test_specular_halfway_to_threshold() {
        // Pointer at x=25 in a 200×100 element: dist_left=25, which is half the threshold
        let (intensity, edge) = specular_intensity(25.0, 50.0, 200.0, 100.0);
        assert!(
            (intensity - 0.5).abs() < f64::EPSILON,
            "Halfway to threshold should have intensity 0.5, got {intensity}"
        );
        assert_eq!(edge, 3, "Nearest edge should be left (3)");
    }

    #[test]
    fn test_specular_top_edge() {
        let (intensity, edge) = specular_intensity(100.0, 10.0, 200.0, 100.0);
        assert_eq!(edge, 0, "Nearest edge should be top (0)");
        let expected = 1.0 - (10.0 / SPECULAR_THRESHOLD_PX);
        assert!(
            (intensity - expected).abs() < 1e-6,
            "Expected {expected}, got {intensity}"
        );
    }

    #[test]
    fn test_specular_right_edge() {
        let (intensity, edge) = specular_intensity(190.0, 50.0, 200.0, 100.0);
        assert_eq!(edge, 1, "Nearest edge should be right (1)");
        let expected = 1.0 - (10.0 / SPECULAR_THRESHOLD_PX);
        assert!(
            (intensity - expected).abs() < 1e-6,
            "Expected {expected}, got {intensity}"
        );
    }

    #[test]
    fn test_specular_bottom_edge() {
        let (intensity, edge) = specular_intensity(100.0, 95.0, 200.0, 100.0);
        assert_eq!(edge, 2, "Nearest edge should be bottom (2)");
        let expected = 1.0 - (5.0 / SPECULAR_THRESHOLD_PX);
        assert!(
            (intensity - expected).abs() < 1e-6,
            "Expected {expected}, got {intensity}"
        );
    }

    #[test]
    fn test_specular_outside_element() {
        let (intensity, _) = specular_intensity(-10.0, 50.0, 200.0, 100.0);
        assert!(
            intensity.abs() < f64::EPSILON,
            "Outside element should be zero intensity"
        );
    }

    #[test]
    fn test_specular_beyond_threshold() {
        // Center of a 400×400 element — 200px from any edge, well beyond 50px threshold
        let (intensity, _) = specular_intensity(200.0, 200.0, 400.0, 400.0);
        assert!(
            intensity.abs() < f64::EPSILON,
            "Beyond threshold should be zero intensity"
        );
    }

    #[test]
    fn test_box_shadow_zero_intensity() {
        let shadow = specular_box_shadow(0.0, 0);
        assert!(shadow.is_empty(), "Zero intensity should produce no shadow");
    }

    #[test]
    fn test_box_shadow_full_intensity() {
        let shadow = specular_box_shadow(1.0, 0);
        assert!(
            shadow.contains("rgba(188, 19, 254,"),
            "Full intensity should contain rgba color"
        );
        assert!(
            shadow.contains("0.400"),
            "Full intensity alpha should be 0.400"
        );
    }

    #[test]
    fn test_box_shadow_directional_offsets() {
        // Top edge: negative Y offset
        let top = specular_box_shadow(1.0, 0);
        assert!(
            top.starts_with("0px -"),
            "Top edge should have negative Y offset"
        );

        // Right edge: positive X offset
        let right = specular_box_shadow(1.0, 1);
        assert!(
            right.starts_with("6px 0px"),
            "Right edge should have positive X offset"
        );

        // Bottom edge: positive Y offset
        let bottom = specular_box_shadow(1.0, 2);
        assert!(
            bottom.starts_with("0px 6px"),
            "Bottom edge should have positive Y offset"
        );

        // Left edge: negative X offset
        let left = specular_box_shadow(1.0, 3);
        assert!(
            left.starts_with("-6px 0px"),
            "Left edge should have negative X offset"
        );
    }

    #[test]
    fn test_design_spec_constants() {
        assert_eq!(COLOR_VOID, "#020203");
        assert_eq!(COLOR_CYBER_PURPLE, "#BC13FE");
        assert_eq!(COLOR_ELECTRIC_CYAN, "#00F2FF");
        assert_eq!(COLOR_EMERALD, "#10B981");
        assert!((SCANLINE_OPACITY - 0.025).abs() < f64::EPSILON as f32);
        assert_eq!(SCANLINE_SPACING, 4);
        assert!((SCANLINE_THICKNESS - 1.0).abs() < f64::EPSILON as f32);
    }
}
