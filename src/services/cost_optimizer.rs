// src/services/cost_optimizer.rs
//! Cost Optimizer
//!
//! Multi-cloud cost optimization and resource allocation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cloud provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Aws,
    Gcp,
    Azure,
    DigitalOcean,
    Hetzner,
    Vultr,
}

impl CloudProvider {
    pub fn name(&self) -> &'static str {
        match self {
            CloudProvider::Aws => "AWS",
            CloudProvider::Gcp => "Google Cloud",
            CloudProvider::Azure => "Azure",
            CloudProvider::DigitalOcean => "DigitalOcean",
            CloudProvider::Hetzner => "Hetzner",
            CloudProvider::Vultr => "Vultr",
        }
    }
}

/// Instance pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancePricing {
    pub provider: CloudProvider,
    pub instance_type: String,
    pub region: String,
    pub vcpus: u8,
    pub memory_gb: u16,
    pub bandwidth_gb: u32,
    pub hourly_cost: f64,
    pub bandwidth_cost_per_gb: f64,
}

impl InstancePricing {
    /// Calculate monthly cost
    pub fn monthly_cost(&self, bandwidth_gb_per_month: u32) -> f64 {
        let compute_cost = self.hourly_cost * 730.0; // ~30 days
        let bandwidth_cost = (bandwidth_gb_per_month as f64 - self.bandwidth_gb as f64).max(0.0)
            * self.bandwidth_cost_per_gb;
        compute_cost + bandwidth_cost
    }

    /// Cost per vCPU per month
    pub fn cost_per_vcpu(&self) -> f64 {
        (self.hourly_cost * 730.0) / self.vcpus as f64
    }

    /// Cost per GB memory per month
    pub fn cost_per_gb_memory(&self) -> f64 {
        (self.hourly_cost * 730.0) / self.memory_gb as f64
    }
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_vcpus: u8,
    pub min_memory_gb: u16,
    pub estimated_bandwidth_gb: u32,
    pub required_regions: Vec<String>,
}

/// Cost optimizer
pub struct CostOptimizer {
    pricing_db: HashMap<String, InstancePricing>,
}

impl CostOptimizer {
    pub fn new() -> Self {
        let mut pricing_db = HashMap::new();

        // Sample pricing data (would be loaded from API in production)
        pricing_db.insert(
            "aws_t3_medium_us_east_1".to_string(),
            InstancePricing {
                provider: CloudProvider::Aws,
                instance_type: "t3.medium".to_string(),
                region: "us-east-1".to_string(),
                vcpus: 2,
                memory_gb: 4,
                bandwidth_gb: 100,
                hourly_cost: 0.0416,
                bandwidth_cost_per_gb: 0.09,
            },
        );

        pricing_db.insert(
            "gcp_e2_medium_us_central1".to_string(),
            InstancePricing {
                provider: CloudProvider::Gcp,
                instance_type: "e2-medium".to_string(),
                region: "us-central1".to_string(),
                vcpus: 2,
                memory_gb: 4,
                bandwidth_gb: 100,
                hourly_cost: 0.0335,
                bandwidth_cost_per_gb: 0.12,
            },
        );

        pricing_db.insert(
            "hetzner_cx21_eu_central".to_string(),
            InstancePricing {
                provider: CloudProvider::Hetzner,
                instance_type: "CX21".to_string(),
                region: "eu-central".to_string(),
                vcpus: 2,
                memory_gb: 4,
                bandwidth_gb: 20000, // 20TB included
                hourly_cost: 0.0068,
                bandwidth_cost_per_gb: 0.01,
            },
        );

        pricing_db.insert(
            "digitalocean_basic_2vcpu_nyc3".to_string(),
            InstancePricing {
                provider: CloudProvider::DigitalOcean,
                instance_type: "Basic 2vCPU".to_string(),
                region: "nyc3".to_string(),
                vcpus: 2,
                memory_gb: 4,
                bandwidth_gb: 3000,
                hourly_cost: 0.0268,
                bandwidth_cost_per_gb: 0.01,
            },
        );

        Self { pricing_db }
    }

    /// Find cheapest option meeting requirements
    pub fn find_cheapest(&self, requirements: &ResourceRequirements) -> Option<CostRecommendation> {
        let mut candidates: Vec<(&String, &InstancePricing, f64)> = self
            .pricing_db
            .iter()
            .filter(|(_, pricing)| {
                pricing.vcpus >= requirements.min_vcpus
                    && pricing.memory_gb >= requirements.min_memory_gb
            })
            .map(|(id, pricing)| {
                let monthly_cost = pricing.monthly_cost(requirements.estimated_bandwidth_gb);
                (id, pricing, monthly_cost)
            })
            .collect();

        candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        candidates.first().map(|(id, pricing, monthly_cost)| {
            CostRecommendation {
                instance_id: (*id).clone(),
                provider: pricing.provider.clone(),
                instance_type: pricing.instance_type.clone(),
                region: pricing.region.clone(),
                monthly_cost: *monthly_cost,
                savings_percent: 0.0, // Would compare to current
                reason: "Lowest cost option meeting requirements".to_string(),
            }
        })
    }

    /// Compare costs across providers
    pub fn compare_providers(
        &self,
        requirements: &ResourceRequirements,
    ) -> Vec<ProviderComparison> {
        let mut comparisons: HashMap<CloudProvider, Vec<f64>> = HashMap::new();

        for pricing in self.pricing_db.values() {
            if pricing.vcpus >= requirements.min_vcpus
                && pricing.memory_gb >= requirements.min_memory_gb
            {
                let monthly_cost = pricing.monthly_cost(requirements.estimated_bandwidth_gb);
                comparisons
                    .entry(pricing.provider.clone())
                    .or_insert_with(Vec::new)
                    .push(monthly_cost);
            }
        }

        let mut result: Vec<ProviderComparison> = comparisons
            .iter()
            .map(|(provider, costs)| {
                let avg_cost = costs.iter().sum::<f64>() / costs.len() as f64;
                let min_cost = costs.iter().cloned().fold(f64::INFINITY, f64::min);
                ProviderComparison {
                    provider: provider.clone(),
                    avg_monthly_cost: avg_cost,
                    min_monthly_cost: min_cost,
                    instance_count: costs.len(),
                }
            })
            .collect();

        result.sort_by(|a, b| a.min_monthly_cost.partial_cmp(&b.min_monthly_cost).unwrap());
        result
    }

    /// Get cost optimization recommendations
    pub fn get_recommendations(
        &self,
        current_spend: f64,
        requirements: &ResourceRequirements,
    ) -> Vec<CostRecommendation> {
        let mut recommendations = Vec::new();

        if let Some(cheapest) = self.find_cheapest(requirements) {
            let savings = current_spend - cheapest.monthly_cost;
            if savings > 0.0 {
                recommendations.push(CostRecommendation {
                    instance_id: cheapest.instance_id,
                    provider: cheapest.provider,
                    instance_type: cheapest.instance_type,
                    region: cheapest.region,
                    monthly_cost: cheapest.monthly_cost,
                    savings_percent: (savings / current_spend) * 100.0,
                    reason: format!(
                        "Save ${:.2}/month ({:.1}%)",
                        savings,
                        (savings / current_spend) * 100.0
                    ),
                });
            }
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecommendation {
    pub instance_id: String,
    pub provider: CloudProvider,
    pub instance_type: String,
    pub region: String,
    pub monthly_cost: f64,
    pub savings_percent: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderComparison {
    pub provider: CloudProvider,
    pub avg_monthly_cost: f64,
    pub min_monthly_cost: f64,
    pub instance_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_pricing_monthly_cost() {
        let pricing = InstancePricing {
            provider: CloudProvider::Aws,
            instance_type: "t3.medium".to_string(),
            region: "us-east-1".to_string(),
            vcpus: 2,
            memory_gb: 4,
            bandwidth_gb: 100,
            hourly_cost: 0.10,
            bandwidth_cost_per_gb: 0.09,
        };

        let monthly_cost = pricing.monthly_cost(200);
        // 0.10 * 730 + (200 - 100) * 0.09 = 73 + 9 = 82
        assert!((monthly_cost - 82.0).abs() < 0.1);
    }

    #[test]
    fn test_find_cheapest() {
        let optimizer = CostOptimizer::new();

        let requirements = ResourceRequirements {
            min_vcpus: 2,
            min_memory_gb: 4,
            estimated_bandwidth_gb: 500,
            required_regions: vec!["us-east-1".to_string()],
        };

        let recommendation = optimizer.find_cheapest(&requirements);
        assert!(recommendation.is_some());

        let rec = recommendation.unwrap();
        // Hetzner should be cheapest
        assert_eq!(rec.provider, CloudProvider::Hetzner);
    }

    #[test]
    fn test_compare_providers() {
        let optimizer = CostOptimizer::new();

        let requirements = ResourceRequirements {
            min_vcpus: 2,
            min_memory_gb: 4,
            estimated_bandwidth_gb: 500,
            required_regions: vec![],
        };

        let comparisons = optimizer.compare_providers(&requirements);
        assert!(!comparisons.is_empty());

        // Should be sorted by min cost
        for i in 1..comparisons.len() {
            assert!(comparisons[i].min_monthly_cost >= comparisons[i - 1].min_monthly_cost);
        }
    }
}
