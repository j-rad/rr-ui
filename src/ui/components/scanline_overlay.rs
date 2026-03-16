//! Scanline Overlay Component
//!
//! A full-screen CRT scanline effect rendered as an inline SVG pattern.
//! Fixed position, pointer-events-none — purely cosmetic texture overlay
//! that adds visual depth to the Obsidian Engine design system.
//!
//! Renders horizontal lines with configurable spacing, thickness, and opacity
//! from the theme constants.

use crate::ui::theme;
use dioxus::prelude::*;

/// Full-screen CRT scanline overlay.
///
/// Places a repeating horizontal line pattern covering the entire viewport
/// at 2.5% opacity. This component should be mounted once at the root level
/// of the application layout.
#[component]
pub fn ScanlineOverlay() -> Element {
    let opacity = theme::SCANLINE_OPACITY;
    let spacing = theme::SCANLINE_SPACING;
    let thickness = theme::SCANLINE_THICKNESS;

    rsx! {
        div {
            class: "fixed inset-0 pointer-events-none z-50",
            style: "opacity: {opacity};",

            svg {
                class: "w-full h-full",
                xmlns: "http://www.w3.org/2000/svg",
                width: "100%",
                height: "100%",

                // Define the repeating scanline pattern
                defs {
                    pattern {
                        id: "edgeray-scanlines",
                        width: "1",
                        height: "{spacing}",
                        pattern_units: "userSpaceOnUse",

                        // Each scanline is a thin white horizontal bar
                        rect {
                            x: "0",
                            y: "0",
                            width: "1",
                            height: "{thickness}",
                            fill: "white",
                        }
                    }
                }

                // Fill the entire viewport with the scanline pattern
                rect {
                    width: "100%",
                    height: "100%",
                    fill: "url(#edgeray-scanlines)",
                }
            }
        }
    }
}

/// Technical grid overlay (2px grid) — even more subtle than scanlines.
///
/// Optional secondary texture for areas that need extra visual depth,
/// such as the Power Core or telemetry panels.
#[component]
pub fn GridOverlay() -> Element {
    let grid_size = theme::GRID_SIZE;
    let grid_opacity = theme::GRID_OPACITY;

    rsx! {
        div {
            class: "absolute inset-0 pointer-events-none",
            style: "opacity: {grid_opacity};",

            svg {
                class: "w-full h-full",
                xmlns: "http://www.w3.org/2000/svg",
                width: "100%",
                height: "100%",

                defs {
                    pattern {
                        id: "edgeray-grid",
                        width: "{grid_size}",
                        height: "{grid_size}",
                        pattern_units: "userSpaceOnUse",

                        // Vertical grid line
                        rect {
                            x: "0",
                            y: "0",
                            width: "0.5",
                            height: "{grid_size}",
                            fill: "white",
                        }

                        // Horizontal grid line
                        rect {
                            x: "0",
                            y: "0",
                            width: "{grid_size}",
                            height: "0.5",
                            fill: "white",
                        }
                    }
                }

                rect {
                    width: "100%",
                    height: "100%",
                    fill: "url(#edgeray-grid)",
                }
            }
        }
    }
}
