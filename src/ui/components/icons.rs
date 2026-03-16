//! Lucide-style SVG Icons
//!
//! Inline SVG icons matching the lucide icon set used in the Svelte frontend.

use dioxus::prelude::*;

/// Icon size prop
#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    #[props(default = 16)]
    pub size: u32,
    #[props(default = "currentColor".to_string())]
    pub color: String,
}

#[component]
pub fn LayoutDashboard(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "3", y: "3", width: "7", height: "9" }
            rect { x: "14", y: "3", width: "7", height: "5" }
            rect { x: "14", y: "12", width: "7", height: "9" }
            rect { x: "3", y: "16", width: "7", height: "5" }
        }
    }
}

#[component]
pub fn ArrowRightLeft(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m16 3 4 4-4 4" }
            path { d: "M20 7H4" }
            path { d: "m8 21-4-4 4-4" }
            path { d: "M4 17h16" }
        }
    }
}

#[component]
pub fn Activity(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 12h-4l-3 9L9 3l-3 9H2" }
        }
    }
}

#[component]
pub fn Settings(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

#[component]
pub fn Cog(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z" }
            path { d: "M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" }
            path { d: "M12 2v2" }
            path { d: "M12 22v-2" }
            path { d: "m17 20.66-1-1.73" }
            path { d: "M11 10.27 7 3.34" }
            path { d: "m20.66 17-1.73-1" }
            path { d: "m3.34 7 1.73 1" }
            path { d: "M14 12h8" }
            path { d: "M2 12h2" }
            path { d: "m20.66 7-1.73 1" }
            path { d: "m3.34 17 1.73-1" }
            path { d: "m17 3.34-1 1.73" }
            path { d: "m11 13.73-4 6.93" }
        }
    }
}

#[component]
pub fn LogOut(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
            polyline { points: "16 17 21 12 16 7" }
            line { x1: "21", x2: "9", y1: "12", y2: "12" }
        }
    }
}

#[component]
pub fn Moon(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" }
        }
    }
}

#[component]
pub fn Sun(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2" }
            path { d: "M12 20v2" }
            path { d: "m4.93 4.93 1.41 1.41" }
            path { d: "m17.66 17.66 1.41 1.41" }
            path { d: "M2 12h2" }
            path { d: "M20 12h2" }
            path { d: "m6.34 17.66-1.41 1.41" }
            path { d: "m19.07 4.93-1.41 1.41" }
        }
    }
}

#[component]
pub fn ChevronLeft(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m15 18-6-6 6-6" }
        }
    }
}

#[component]
pub fn ChevronRight(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m9 18 6-6-6-6" }
        }
    }
}

#[component]
pub fn FileText(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" }
            polyline { points: "14 2 14 8 20 8" }
            line { x1: "16", x2: "8", y1: "13", y2: "13" }
            line { x1: "16", x2: "8", y1: "17", y2: "17" }
            line { x1: "10", x2: "8", y1: "9", y2: "9" }
        }
    }
}

#[component]
pub fn Database(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            ellipse { cx: "12", cy: "5", rx: "9", ry: "3" }
            path { d: "M3 5V19A9 3 0 0 0 21 19V5" }
            path { d: "M3 12A9 3 0 0 0 21 12" }
        }
    }
}

#[component]
pub fn X(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        }
    }
}

#[component]
pub fn CheckCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
            path { d: "m9 11 3 3L22 4" }
        }
    }
}

#[component]
pub fn XCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "m15 9-6 6" }
            path { d: "m9 9 6 6" }
        }
    }
}

#[component]
pub fn AlertTriangle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" }
            path { d: "M12 9v4" }
            path { d: "M12 17h.01" }
        }
    }
}

#[component]
pub fn Info(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M12 16v-4" }
            path { d: "M12 8h.01" }
        }
    }
}
