//! Statistic Card Component
//!
//! Reusable statistics display card with animated value transitions.
//! Matches 3x-ui dashboard aesthetics with Ant Design styling.

use dioxus::prelude::*;

/// Trend direction for statistic values
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TrendDirection {
    Up,
    Down,
    #[default]
    Neutral,
}

/// Color variants for statistic cards
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum StatisticColor {
    #[default]
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

impl StatisticColor {
    fn bg_class(&self) -> &'static str {
        match self {
            Self::Primary => "bg-cyan-500/10",
            Self::Success => "bg-emerald-500/10",
            Self::Warning => "bg-amber-500/10",
            Self::Danger => "bg-rose-500/10",
            Self::Info => "bg-violet-500/10",
        }
    }

    fn icon_class(&self) -> &'static str {
        match self {
            Self::Primary => "text-cyan-400",
            Self::Success => "text-emerald-400",
            Self::Warning => "text-amber-400",
            Self::Danger => "text-rose-400",
            Self::Info => "text-violet-400",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct StatisticCardProps {
    /// The label/title for this statistic
    pub title: String,
    /// The main value to display
    pub value: String,
    /// Material icon name (optional)
    #[props(default)]
    pub icon: Option<String>,
    /// Suffix text (e.g., "GB", "%")
    #[props(default)]
    pub suffix: Option<String>,
    /// Prefix text (e.g., "$")
    #[props(default)]
    pub prefix: Option<String>,
    /// Trend direction for the value
    #[props(default)]
    pub trend: TrendDirection,
    /// Trend value (e.g., "+5%")
    #[props(default)]
    pub trend_value: Option<String>,
    /// Color variant
    #[props(default)]
    pub color: StatisticColor,
    /// Whether to use compact layout
    #[props(default = false)]
    pub compact: bool,
}

/// A statistic display card with icon, value, and trend indicator
#[component]
pub fn StatisticCard(props: StatisticCardProps) -> Element {
    let trend_class = match props.trend {
        TrendDirection::Up => "text-green-400",
        TrendDirection::Down => "text-red-400",
        TrendDirection::Neutral => "text-gray-400",
    };

    let trend_icon = match props.trend {
        TrendDirection::Up => "trending_up",
        TrendDirection::Down => "trending_down",
        TrendDirection::Neutral => "trending_flat",
    };

    if props.compact {
        // Compact inline layout
        rsx! {
            div { class: "flex items-center gap-3",
                if let Some(ref icon) = props.icon {
                    div { class: "p-2 rounded-lg {props.color.bg_class()}",
                        span { class: "material-symbols-outlined text-[20px] {props.color.icon_class()}", "{icon}" }
                    }
                }
                div {
                    div { class: "text-xs text-gray-500 uppercase tracking-wider", "{props.title}" }
                    div { class: "text-lg font-bold text-white",
                        if let Some(ref prefix) = props.prefix {
                            span { "{prefix}" }
                        }
                        span { "{props.value}" }
                        if let Some(ref suffix) = props.suffix {
                            span { class: "text-sm text-gray-400 ml-1", "{suffix}" }
                        }
                    }
                }
            }
        }
    } else {
        // Full card layout
        rsx! {
            div { class: "relative bg-glass-bg/60 backdrop-blur-xl border border-glass-border rounded-xl p-5 transition-all duration-300 hover:border-primary/30 hover:-translate-y-0.5 hover:shadow-glow overflow-hidden group",
                // Gradient overlay
                div { class: "absolute inset-0 bg-gradient-to-br from-white/[0.02] via-transparent to-black/[0.02] pointer-events-none rounded-xl" }

                div { class: "relative flex items-start justify-between",
                    div {
                        div { class: "text-sm text-text-secondary mb-2 font-medium", "{props.title}" }
                        div { class: "text-2xl font-bold text-text-main tracking-tight animate-counter",
                            if let Some(ref prefix) = props.prefix {
                                span { "{prefix}" }
                            }
                            span { class: "statistic-value", "{props.value}" }
                            if let Some(ref suffix) = props.suffix {
                                span { class: "text-base text-text-muted ml-1.5 font-medium", "{suffix}" }
                            }
                        }
                        if let Some(ref trend_val) = props.trend_value {
                            div { class: "flex items-center gap-1.5 mt-2.5 text-sm {trend_class}",
                                span { class: "material-symbols-outlined text-[16px]", "{trend_icon}" }
                                span { class: "font-medium", "{trend_val}" }
                            }
                        }
                    }
                    if let Some(ref icon) = props.icon {
                        div { class: "p-3 rounded-xl {props.color.bg_class()} transition-transform group-hover:scale-110",
                            span { class: "material-symbols-outlined text-[28px] {props.color.icon_class()}", "{icon}" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistic_color_classes() {
        assert_eq!(StatisticColor::Primary.bg_class(), "bg-cyan-500/10");
        assert_eq!(StatisticColor::Success.icon_class(), "text-emerald-400");
    }

    #[test]
    fn test_trend_direction_default() {
        let trend = TrendDirection::default();
        assert_eq!(trend, TrendDirection::Neutral);
    }
}
