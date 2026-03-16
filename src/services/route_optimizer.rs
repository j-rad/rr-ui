// src/services/route_optimizer.rs
//! Route Optimizer
//!
//! ML-based intelligent routing with latency prediction

use crate::domain::models::MeshNode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Route metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMetrics {
    pub route_id: String,
    pub source_node: String,
    pub dest_node: String,
    pub avg_latency_ms: f64,
    pub packet_loss_percent: f64,
    pub bandwidth_mbps: f64,
    pub cost_per_gb: f64,
    pub reliability_score: f64,
}

impl RouteMetrics {
    /// Calculate overall route score (0.0 - 1.0, higher is better)
    pub fn score(&self) -> f64 {
        let latency_score = (1.0 - (self.avg_latency_ms / 500.0).min(1.0)) * 0.4;
        let loss_score = (1.0 - (self.packet_loss_percent / 5.0).min(1.0)) * 0.3;
        let bandwidth_score = (self.bandwidth_mbps / 1000.0).min(1.0) * 0.2;
        let cost_score = (1.0 - (self.cost_per_gb / 0.10).min(1.0)) * 0.1;

        latency_score + loss_score + bandwidth_score + cost_score
    }
}

/// Historical route data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDataPoint {
    pub timestamp: i64,
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub throughput_mbps: f64,
}

/// Simple linear regression model
#[derive(Debug, Clone)]
pub struct LinearModel {
    slope: f64,
    intercept: f64,
}

impl LinearModel {
    /// Train model from data points
    pub fn train(data: &[(f64, f64)]) -> Self {
        if data.is_empty() {
            return Self {
                slope: 0.0,
                intercept: 0.0,
            };
        }

        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        Self { slope, intercept }
    }

    /// Predict value
    pub fn predict(&self, x: f64) -> f64 {
        self.slope * x + self.intercept
    }
}

/// Route optimizer
pub struct RouteOptimizer {
    routes: HashMap<String, Vec<RouteDataPoint>>,
    current_metrics: HashMap<String, RouteMetrics>,
    prediction_window_hours: u64,
}

impl RouteOptimizer {
    pub fn new(prediction_window_hours: u64) -> Self {
        Self {
            routes: HashMap::new(),
            current_metrics: HashMap::new(),
            prediction_window_hours,
        }
    }

    /// Generate an Anycast Load Balanced subscription link
    ///
    /// Returns a subscription link pointing to the node with the lowest current CPU load.
    pub fn generate_optimal_subscription_link(
        &self,
        active_nodes: &[MeshNode],
        user_id: &str,
    ) -> Option<String> {
        let best_node = active_nodes.iter().min_by(|a, b| {
            a.health
                .cpu_percent
                .partial_cmp(&b.health.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;

        // Format: https://{address}:{port}/sub/{user_id}
        Some(format!(
            "https://{}:{}/sub/{}",
            best_node.address, best_node.port, user_id
        ))
    }

    /// Record route measurement
    pub fn record_measurement(&mut self, route_id: String, data: RouteDataPoint) {
        self.routes
            .entry(route_id.clone())
            .or_insert_with(Vec::new)
            .push(data);

        // Keep only last 1000 measurements
        if let Some(history) = self.routes.get_mut(&route_id) {
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// Update current route metrics
    pub fn update_metrics(&mut self, route_id: String, metrics: RouteMetrics) {
        self.current_metrics.insert(route_id, metrics);
    }

    /// Predict latency for next N hours
    pub fn predict_latency(&self, route_id: &str, hours_ahead: u64) -> Option<f64> {
        let history = self.routes.get(route_id)?;
        if history.len() < 10 {
            return None;
        }

        // Prepare training data (timestamp -> latency)
        let data: Vec<(f64, f64)> = history
            .iter()
            .map(|dp| (dp.timestamp as f64, dp.latency_ms))
            .collect();

        let model = LinearModel::train(&data);
        let future_timestamp = chrono::Utc::now().timestamp() + (hours_ahead as i64 * 3600);
        Some(model.predict(future_timestamp as f64))
    }

    /// Select optimal route based on current metrics and predictions
    pub fn select_optimal_route(&self, available_routes: &[String]) -> Option<String> {
        let mut best_route: Option<(String, f64)> = None;

        for route_id in available_routes {
            if let Some(metrics) = self.current_metrics.get(route_id) {
                let mut score = metrics.score();

                // Adjust score based on predicted latency trend
                if let Some(predicted_latency) = self.predict_latency(route_id, 1) {
                    let current_latency = metrics.avg_latency_ms;
                    if predicted_latency > current_latency * 1.2 {
                        // Latency trending up - penalize
                        score *= 0.8;
                    } else if predicted_latency < current_latency * 0.8 {
                        // Latency trending down - bonus
                        score *= 1.2;
                    }
                }

                if let Some((_, best_score)) = &best_route {
                    if score > *best_score {
                        best_route = Some((route_id.clone(), score));
                    }
                } else {
                    best_route = Some((route_id.clone(), score));
                }
            }
        }

        best_route.map(|(route_id, _)| route_id)
    }

    /// Get route recommendations
    pub fn get_recommendations(&self) -> Vec<RouteRecommendation> {
        let mut recommendations = Vec::new();

        for (route_id, metrics) in &self.current_metrics {
            // High latency warning
            if metrics.avg_latency_ms > 200.0 {
                recommendations.push(RouteRecommendation {
                    route_id: route_id.clone(),
                    severity: RecommendationSeverity::Warning,
                    message: format!(
                        "High latency detected: {:.1}ms (threshold: 200ms)",
                        metrics.avg_latency_ms
                    ),
                    action: "Consider switching to alternative route".to_string(),
                });
            }

            // High packet loss warning
            if metrics.packet_loss_percent > 1.0 {
                recommendations.push(RouteRecommendation {
                    route_id: route_id.clone(),
                    severity: RecommendationSeverity::Critical,
                    message: format!(
                        "High packet loss: {:.2}% (threshold: 1%)",
                        metrics.packet_loss_percent
                    ),
                    action: "Immediate route change recommended".to_string(),
                });
            }

            // Cost optimization
            if metrics.cost_per_gb > 0.05 {
                recommendations.push(RouteRecommendation {
                    route_id: route_id.clone(),
                    severity: RecommendationSeverity::Info,
                    message: format!("High cost route: ${:.4}/GB", metrics.cost_per_gb),
                    action: "Consider cheaper alternatives if available".to_string(),
                });
            }
        }

        recommendations
    }

    /// Get route statistics
    pub fn get_route_stats(&self, route_id: &str) -> Option<RouteStats> {
        let history = self.routes.get(route_id)?;
        if history.is_empty() {
            return None;
        }

        let latencies: Vec<f64> = history.iter().map(|dp| dp.latency_ms).collect();
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let min_latency = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_latency = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Some(RouteStats {
            route_id: route_id.to_string(),
            sample_count: history.len(),
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            predicted_latency_1h: self.predict_latency(route_id, 1),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRecommendation {
    pub route_id: String,
    pub severity: RecommendationSeverity,
    pub message: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStats {
    pub route_id: String,
    pub sample_count: usize,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub predicted_latency_1h: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_metrics_score() {
        let metrics = RouteMetrics {
            route_id: "route1".to_string(),
            source_node: "node1".to_string(),
            dest_node: "node2".to_string(),
            avg_latency_ms: 50.0,
            packet_loss_percent: 0.1,
            bandwidth_mbps: 500.0,
            cost_per_gb: 0.02,
            reliability_score: 0.99,
        };

        let score = metrics.score();
        assert!(score > 0.8); // Good route should score high
    }

    #[test]
    fn test_linear_model() {
        let data = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let model = LinearModel::train(&data);

        assert!((model.predict(4.0) - 8.0).abs() < 0.1);
    }

    #[test]
    fn test_route_optimizer() {
        let mut optimizer = RouteOptimizer::new(24);

        let metrics = RouteMetrics {
            route_id: "route1".to_string(),
            source_node: "node1".to_string(),
            dest_node: "node2".to_string(),
            avg_latency_ms: 50.0,
            packet_loss_percent: 0.1,
            bandwidth_mbps: 500.0,
            cost_per_gb: 0.02,
            reliability_score: 0.99,
        };

        optimizer.update_metrics("route1".to_string(), metrics);

        let selected = optimizer.select_optimal_route(&["route1".to_string()]);
        assert_eq!(selected, Some("route1".to_string()));
    }

    #[test]
    fn test_recommendations() {
        let mut optimizer = RouteOptimizer::new(24);

        let bad_metrics = RouteMetrics {
            route_id: "route1".to_string(),
            source_node: "node1".to_string(),
            dest_node: "node2".to_string(),
            avg_latency_ms: 300.0,    // High latency
            packet_loss_percent: 2.0, // High loss
            bandwidth_mbps: 100.0,
            cost_per_gb: 0.08, // High cost
            reliability_score: 0.5,
        };

        optimizer.update_metrics("route1".to_string(), bad_metrics);

        let recommendations = optimizer.get_recommendations();
        assert!(!recommendations.is_empty());
        assert!(
            recommendations
                .iter()
                .any(|r| matches!(r.severity, RecommendationSeverity::Critical))
        );
    }
}
