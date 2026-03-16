use crate::ui::components::connection_matrix::ConnectionMatrix;
use crate::ui::components::sparkline::{DetailSparkline, format_rate};
use crate::ui::components::statistic_card::{StatisticCard, TrendDirection};
use crate::ui::server_fns::*;
use dioxus::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;

#[component]
pub fn ConnectionsPage() -> Element {
    let mut history_data = use_signal(VecDeque::<(i64, i64)>::new);
    let mut total_up = use_signal(|| 0i64);
    let mut total_down = use_signal(|| 0i64);

    // Polling for real-time telemetry
    use_effect(move || {
        spawn(async move {
            loop {
                // Fetch stats for aggregate counters
                if let Ok(stats) = get_traffic_stats().await {
                    let mut up = 0;
                    let mut down = 0;
                    for s in stats {
                        if s.name == "uplink" {
                            up = s.value;
                        } else if s.name == "downlink" {
                            down = s.value;
                        }
                    }
                    total_up.set(up);
                    total_down.set(down);
                }

                // Fetch history for sparklines
                if let Ok(history) = get_traffic_history().await {
                    let mut points = VecDeque::new();
                    for p in history {
                        points.push_back((p.up_rate as i64, p.down_rate as i64));
                    }
                    history_data.set(points);
                }

                #[cfg(feature = "web")]
                crate::ui::sleep::sleep(1500 as u64).await;
                #[cfg(not(feature = "web"))]
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
        });
    });

    let up = total_up.read();
    let down = total_down.read();
    let total_rate = format_rate(*up + *down);

    rsx! {
        div { class: "h-full flex flex-col gap-6 p-6 overflow-hidden",
            // Header
            div { class: "flex justify-between items-end",
                div {
                    h1 { class: "text-3xl font-bold text-white tracking-tight", "NOC Live View" }
                    p { class: "text-gray-400 text-sm mt-1", "Real-time network telemetry and session tracking" }
                }
                div { class: "flex gap-2",
                   // Action buttons?
                }
            }

            // KPIs & Sparklines
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                StatisticCard {
                    title: "Total Throughput".to_string(),
                    value: "{total_rate}",
                    icon: "fas fa-tachometer-alt".to_string(),
                    trend: TrendDirection::Neutral,
                }

                div { class: "md:col-span-2 bg-white/5 backdrop-blur-md rounded-xl p-4 border border-white/10",
                    DetailSparkline {
                        data: history_data.read().clone(),
                        upload_rate: Some(*up),
                        download_rate: Some(*down),
                    }
                }
            }

            // Connection Matrix (Takes remaining height)
            div { class: "flex-1 min-h-0 rounded-xl overflow-hidden shadow-2xl shadow-black/50 ring-1 ring-white/10",
                ConnectionMatrix {}
            }
        }
    }
}
