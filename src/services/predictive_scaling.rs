// src/services/predictive_scaling.rs
//! Predictive Auto-Scaling
//!
//! Forecasts traffic patterns and scales resources proactively

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Traffic pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPattern {
    pub timestamp: i64,
    pub active_connections: u32,
    pub bandwidth_mbps: f64,
    pub cpu_percent: f32,
    pub memory_percent: f32,
}

/// Scaling recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    pub action: ScalingAction,
    pub reason: String,
    pub confidence: f32,
    pub estimated_time_to_threshold: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScalingAction {
    ScaleUp { target_nodes: u32 },
    ScaleDown { target_nodes: u32 },
    NoAction,
}

/// Time-series forecaster
pub struct TimeSeriesForecaster {
    history: VecDeque<f64>,
    max_history: usize,
}

impl TimeSeriesForecaster {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Add data point
    pub fn add_point(&mut self, value: f64) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(value);
    }

    /// Simple moving average forecast
    pub fn forecast_sma(&self, _periods_ahead: usize) -> Option<f64> {
        if self.history.len() < 3 {
            return None;
        }

        let window_size = (self.history.len() / 3).max(3);
        let recent: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(window_size)
            .copied()
            .collect();
        let avg = recent.iter().sum::<f64>() / recent.len() as f64;

        Some(avg)
    }

    /// Exponential moving average forecast
    pub fn forecast_ema(&self, alpha: f64) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }

        let mut ema = self.history[0];
        for &value in self.history.iter().skip(1) {
            ema = alpha * value + (1.0 - alpha) * ema;
        }

        Some(ema)
    }

    /// Detect trend (positive, negative, or neutral)
    pub fn detect_trend(&self) -> Trend {
        if self.history.len() < 10 {
            return Trend::Neutral;
        }

        let recent_half = self.history.len() / 2;
        let first_half: f64 =
            self.history.iter().take(recent_half).sum::<f64>() / recent_half as f64;
        let second_half: f64 = self.history.iter().skip(recent_half).sum::<f64>()
            / (self.history.len() - recent_half) as f64;

        let change_percent = ((second_half - first_half) / first_half) * 100.0;

        if change_percent > 10.0 {
            Trend::Increasing
        } else if change_percent < -10.0 {
            Trend::Decreasing
        } else {
            Trend::Neutral
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Trend {
    #[default]
    Neutral,
    Increasing,
    Decreasing,
}

/// Predictive scaler
pub struct PredictiveScaler {
    connection_forecaster: TimeSeriesForecaster,
    cpu_forecaster: TimeSeriesForecaster,
    memory_forecaster: TimeSeriesForecaster,
    current_nodes: u32,
    min_nodes: u32,
    max_nodes: u32,
    scale_up_threshold: f32,
    scale_down_threshold: f32,
}

impl PredictiveScaler {
    pub fn new(
        current_nodes: u32,
        min_nodes: u32,
        max_nodes: u32,
        scale_up_threshold: f32,
        scale_down_threshold: f32,
    ) -> Self {
        Self {
            connection_forecaster: TimeSeriesForecaster::new(100),
            cpu_forecaster: TimeSeriesForecaster::new(100),
            memory_forecaster: TimeSeriesForecaster::new(100),
            current_nodes,
            min_nodes,
            max_nodes,
            scale_up_threshold,
            scale_down_threshold,
        }
    }

    /// Record traffic pattern
    pub fn record_pattern(&mut self, pattern: TrafficPattern) {
        self.connection_forecaster
            .add_point(pattern.active_connections as f64);
        self.cpu_forecaster.add_point(pattern.cpu_percent as f64);
        self.memory_forecaster
            .add_point(pattern.memory_percent as f64);
    }

    /// Get scaling recommendation
    pub fn get_recommendation(&self) -> ScalingRecommendation {
        // Forecast CPU usage
        let predicted_cpu = self.cpu_forecaster.forecast_ema(0.3).unwrap_or(0.0) as f32;
        let cpu_trend = self.cpu_forecaster.detect_trend();

        // Forecast memory usage
        let predicted_memory = self.memory_forecaster.forecast_ema(0.3).unwrap_or(0.0) as f32;
        let memory_trend = self.memory_forecaster.detect_trend();

        // Determine action
        let (action, reason, confidence) = if predicted_cpu > self.scale_up_threshold
            || predicted_memory > self.scale_up_threshold
        {
            let target = (self.current_nodes + 1).min(self.max_nodes);
            (
                ScalingAction::ScaleUp {
                    target_nodes: target,
                },
                format!(
                    "Predicted resource usage exceeds threshold (CPU: {:.1}%, Memory: {:.1}%)",
                    predicted_cpu, predicted_memory
                ),
                0.8,
            )
        } else if predicted_cpu < self.scale_down_threshold
            && predicted_memory < self.scale_down_threshold
            && self.current_nodes > self.min_nodes
        {
            // Only scale down if trend is decreasing
            if matches!(cpu_trend, Trend::Decreasing) && matches!(memory_trend, Trend::Decreasing) {
                let target = (self.current_nodes - 1).max(self.min_nodes);
                (
                    ScalingAction::ScaleDown {
                        target_nodes: target,
                    },
                    format!(
                        "Resource usage trending down (CPU: {:.1}%, Memory: {:.1}%)",
                        predicted_cpu, predicted_memory
                    ),
                    0.7,
                )
            } else {
                (
                    ScalingAction::NoAction,
                    "Resource usage low but trend not stable".to_string(),
                    0.5,
                )
            }
        } else {
            (
                ScalingAction::NoAction,
                "Resource usage within normal range".to_string(),
                0.9,
            )
        };

        ScalingRecommendation {
            action,
            reason,
            confidence,
            estimated_time_to_threshold: None,
        }
    }

    /// Update current node count
    pub fn update_node_count(&mut self, count: u32) {
        self.current_nodes = count;
    }

    /// Get forecast summary
    pub fn get_forecast_summary(&self) -> ForecastSummary {
        ForecastSummary {
            predicted_cpu: self.cpu_forecaster.forecast_ema(0.3).unwrap_or(0.0) as f32,
            predicted_memory: self.memory_forecaster.forecast_ema(0.3).unwrap_or(0.0) as f32,
            predicted_connections: self.connection_forecaster.forecast_ema(0.3).unwrap_or(0.0)
                as u32,
            cpu_trend: self.cpu_forecaster.detect_trend(),
            memory_trend: self.memory_forecaster.detect_trend(),
            current_nodes: self.current_nodes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSummary {
    pub predicted_cpu: f32,
    pub predicted_memory: f32,
    pub predicted_connections: u32,
    #[serde(skip)]
    pub cpu_trend: Trend,
    #[serde(skip)]
    pub memory_trend: Trend,
    pub current_nodes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forecaster_sma() {
        let mut forecaster = TimeSeriesForecaster::new(10);

        for i in 1..=10 {
            forecaster.add_point(i as f64);
        }

        let forecast = forecaster.forecast_sma(1);
        assert!(forecast.is_some());
        assert!(forecast.unwrap() > 5.0);
    }

    #[test]
    fn test_forecaster_ema() {
        let mut forecaster = TimeSeriesForecaster::new(10);

        for i in 1..=10 {
            forecaster.add_point(i as f64);
        }

        let forecast = forecaster.forecast_ema(0.3);
        assert!(forecast.is_some());
    }

    #[test]
    fn test_trend_detection() {
        let mut forecaster = TimeSeriesForecaster::new(20);

        // Increasing trend
        for i in 1..=20 {
            forecaster.add_point(i as f64);
        }

        assert_eq!(forecaster.detect_trend(), Trend::Increasing);
    }

    #[test]
    fn test_scale_up_recommendation() {
        let mut scaler = PredictiveScaler::new(2, 1, 10, 80.0, 30.0);

        // Simulate high load
        for _ in 0..10 {
            scaler.record_pattern(TrafficPattern {
                timestamp: chrono::Utc::now().timestamp(),
                active_connections: 1000,
                bandwidth_mbps: 500.0,
                cpu_percent: 85.0,
                memory_percent: 75.0,
            });
        }

        let recommendation = scaler.get_recommendation();
        assert!(matches!(
            recommendation.action,
            ScalingAction::ScaleUp { .. }
        ));
    }

    #[test]
    fn test_scale_down_recommendation() {
        let mut scaler = PredictiveScaler::new(5, 1, 10, 80.0, 30.0);

        // Simulate low load with decreasing trend
        for i in (1..=20).rev() {
            scaler.record_pattern(TrafficPattern {
                timestamp: chrono::Utc::now().timestamp(),
                active_connections: 100,
                bandwidth_mbps: 50.0,
                cpu_percent: i as f32,
                memory_percent: i as f32,
            });
        }

        let recommendation = scaler.get_recommendation();
        // Should recommend scale down or no action
        assert!(!matches!(
            recommendation.action,
            ScalingAction::ScaleUp { .. }
        ));
    }
}
