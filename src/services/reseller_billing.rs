// src/services/reseller_billing.rs
//! Reseller Billing System
//!
//! Manages subscription tiers, Stripe integration, and usage tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Subscription tier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Starter,
    Professional,
    Enterprise,
    Custom,
}

impl SubscriptionTier {
    pub fn monthly_price_usd(&self) -> f64 {
        match self {
            SubscriptionTier::Starter => 29.99,
            SubscriptionTier::Professional => 99.99,
            SubscriptionTier::Enterprise => 299.99,
            SubscriptionTier::Custom => 0.0, // Negotiated
        }
    }

    pub fn max_nodes(&self) -> u32 {
        match self {
            SubscriptionTier::Starter => 2,
            SubscriptionTier::Professional => 10,
            SubscriptionTier::Enterprise => 50,
            SubscriptionTier::Custom => u32::MAX,
        }
    }

    pub fn max_users_per_node(&self) -> u32 {
        match self {
            SubscriptionTier::Starter => 100,
            SubscriptionTier::Professional => 500,
            SubscriptionTier::Enterprise => 2000,
            SubscriptionTier::Custom => u32::MAX,
        }
    }

    pub fn features(&self) -> Vec<&'static str> {
        match self {
            SubscriptionTier::Starter => vec![
                "Up to 2 nodes",
                "100 users per node",
                "Basic support",
                "Community plugins",
            ],
            SubscriptionTier::Professional => vec![
                "Up to 10 nodes",
                "500 users per node",
                "Priority support",
                "Advanced analytics",
                "Custom branding",
            ],
            SubscriptionTier::Enterprise => vec![
                "Up to 50 nodes",
                "2000 users per node",
                "24/7 support",
                "SLA guarantee",
                "Custom integrations",
                "Dedicated account manager",
            ],
            SubscriptionTier::Custom => vec![
                "Unlimited nodes",
                "Unlimited users",
                "Custom SLA",
                "White-label solution",
            ],
        }
    }
}

/// Reseller subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerSubscription {
    pub reseller_id: String,
    pub tier: SubscriptionTier,
    pub status: SubscriptionStatus,
    pub billing_cycle: BillingCycle,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub stripe_subscription_id: Option<String>,
    pub usage: UsageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    PastDue,
    Canceled,
    Unpaid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillingCycle {
    Monthly,
    Yearly,
}

impl BillingCycle {
    pub fn discount_percent(&self) -> f64 {
        match self {
            BillingCycle::Monthly => 0.0,
            BillingCycle::Yearly => 20.0, // 20% discount for annual
        }
    }
}

/// Usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageMetrics {
    pub active_nodes: u32,
    pub total_users: u32,
    pub bandwidth_gb: f64,
    pub api_calls: u64,
}

impl UsageMetrics {
    pub fn is_within_limits(&self, tier: &SubscriptionTier) -> bool {
        self.active_nodes <= tier.max_nodes()
            && self.total_users <= (tier.max_users_per_node() * self.active_nodes)
    }

    pub fn overage_charges(&self, tier: &SubscriptionTier) -> f64 {
        let mut charges = 0.0;

        // Node overage: $10/node/month
        if self.active_nodes > tier.max_nodes() {
            let overage = self.active_nodes - tier.max_nodes();
            charges += overage as f64 * 10.0;
        }

        // User overage: $0.10/user/month
        let max_users = tier.max_users_per_node() * tier.max_nodes();
        if self.total_users > max_users {
            let overage = self.total_users - max_users;
            charges += overage as f64 * 0.10;
        }

        charges
    }
}

/// Billing manager
#[cfg(feature = "server")]
pub struct BillingManager {
    subscriptions: HashMap<String, ResellerSubscription>,
    stripe_api_key: String,
}

#[cfg(feature = "server")]
impl BillingManager {
    pub fn new(stripe_api_key: String) -> Self {
        Self {
            subscriptions: HashMap::new(),
            stripe_api_key,
        }
    }

    /// Create new subscription
    pub async fn create_subscription(
        &mut self,
        reseller_id: String,
        tier: SubscriptionTier,
        billing_cycle: BillingCycle,
    ) -> Result<ResellerSubscription, String> {
        let now = chrono::Utc::now().timestamp();
        let period_end = match billing_cycle {
            BillingCycle::Monthly => now + 30 * 86400,
            BillingCycle::Yearly => now + 365 * 86400,
        };

        let subscription = ResellerSubscription {
            reseller_id: reseller_id.clone(),
            tier,
            status: SubscriptionStatus::Trialing, // 14-day trial
            billing_cycle,
            current_period_start: now,
            current_period_end: period_end,
            stripe_subscription_id: None,
            usage: UsageMetrics::default(),
        };

        self.subscriptions.insert(reseller_id, subscription.clone());

        Ok(subscription)
    }

    /// Update usage metrics
    pub fn update_usage(&mut self, reseller_id: &str, usage: UsageMetrics) {
        if let Some(subscription) = self.subscriptions.get_mut(reseller_id) {
            subscription.usage = usage.clone();

            // Check if over limits
            if !usage.is_within_limits(&subscription.tier) {
                log::warn!("Reseller {} exceeded limits: {:?}", reseller_id, usage);
            }
        }
    }

    /// Calculate invoice
    pub fn calculate_invoice(&self, reseller_id: &str) -> Option<Invoice> {
        let subscription = self.subscriptions.get(reseller_id)?;

        let base_price = subscription.tier.monthly_price_usd();
        let discount = base_price * (subscription.billing_cycle.discount_percent() / 100.0);
        let overage = subscription.usage.overage_charges(&subscription.tier);

        let subtotal = base_price - discount + overage;
        let tax = subtotal * 0.08; // 8% tax
        let total = subtotal + tax;

        Some(Invoice {
            reseller_id: reseller_id.to_string(),
            period_start: subscription.current_period_start,
            period_end: subscription.current_period_end,
            base_price,
            discount,
            overage_charges: overage,
            subtotal,
            tax,
            total,
            line_items: vec![
                LineItem {
                    description: format!("{:?} Plan", subscription.tier),
                    amount: base_price,
                },
                LineItem {
                    description: "Annual discount".to_string(),
                    amount: -discount,
                },
                LineItem {
                    description: "Overage charges".to_string(),
                    amount: overage,
                },
            ],
        })
    }

    /// Process payment via Stripe
    pub async fn process_payment(&self, reseller_id: &str) -> Result<PaymentResult, String> {
        let invoice = self
            .calculate_invoice(reseller_id)
            .ok_or_else(|| "Subscription not found".to_string())?;

        // In production, would call Stripe API
        // For now, simulate success
        Ok(PaymentResult {
            success: true,
            transaction_id: format!("txn_{}", chrono::Utc::now().timestamp()),
            amount: invoice.total,
            currency: "USD".to_string(),
        })
    }

    /// Cancel subscription
    pub fn cancel_subscription(&mut self, reseller_id: &str) -> Result<(), String> {
        if let Some(subscription) = self.subscriptions.get_mut(reseller_id) {
            subscription.status = SubscriptionStatus::Canceled;
            Ok(())
        } else {
            Err("Subscription not found".to_string())
        }
    }

    /// Get subscription
    pub fn get_subscription(&self, reseller_id: &str) -> Option<&ResellerSubscription> {
        self.subscriptions.get(reseller_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub reseller_id: String,
    pub period_start: i64,
    pub period_end: i64,
    pub base_price: f64,
    pub discount: f64,
    pub overage_charges: f64,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub line_items: Vec<LineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub success: bool,
    pub transaction_id: String,
    pub amount: f64,
    pub currency: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_pricing() {
        assert_eq!(SubscriptionTier::Starter.monthly_price_usd(), 29.99);
        assert_eq!(SubscriptionTier::Professional.monthly_price_usd(), 99.99);
        assert_eq!(SubscriptionTier::Enterprise.monthly_price_usd(), 299.99);
    }

    #[test]
    fn test_tier_limits() {
        let starter = SubscriptionTier::Starter;
        assert_eq!(starter.max_nodes(), 2);
        assert_eq!(starter.max_users_per_node(), 100);
    }

    #[test]
    fn test_usage_within_limits() {
        let usage = UsageMetrics {
            active_nodes: 1,
            total_users: 50,
            bandwidth_gb: 100.0,
            api_calls: 1000,
        };

        assert!(usage.is_within_limits(&SubscriptionTier::Starter));
    }

    #[test]
    fn test_overage_charges() {
        let usage = UsageMetrics {
            active_nodes: 5,  // 3 over limit for Starter (max 2)
            total_users: 250, // 50 over limit (max 200 for 2 nodes)
            bandwidth_gb: 1000.0,
            api_calls: 10000,
        };

        let overage = usage.overage_charges(&SubscriptionTier::Starter);
        // 3 nodes * $10 + 50 users * $0.10 = $35
        assert_eq!(overage, 35.0);
    }

    #[test]
    fn test_annual_discount() {
        let monthly = BillingCycle::Monthly;
        let yearly = BillingCycle::Yearly;

        assert_eq!(monthly.discount_percent(), 0.0);
        assert_eq!(yearly.discount_percent(), 20.0);
    }

    #[tokio::test]
    async fn test_subscription_creation() {
        let mut manager = BillingManager::new("test_key".to_string());

        let subscription = manager
            .create_subscription(
                "reseller1".to_string(),
                SubscriptionTier::Professional,
                BillingCycle::Monthly,
            )
            .await
            .unwrap();

        assert_eq!(subscription.tier, SubscriptionTier::Professional);
        assert_eq!(subscription.status, SubscriptionStatus::Trialing);
    }

    #[test]
    fn test_invoice_calculation() {
        let mut manager = BillingManager::new("test_key".to_string());

        let subscription = ResellerSubscription {
            reseller_id: "reseller1".to_string(),
            tier: SubscriptionTier::Professional,
            status: SubscriptionStatus::Active,
            billing_cycle: BillingCycle::Monthly,
            current_period_start: 0,
            current_period_end: 2592000,
            stripe_subscription_id: None,
            usage: UsageMetrics {
                active_nodes: 12, // 2 over limit
                total_users: 500,
                bandwidth_gb: 1000.0,
                api_calls: 10000,
            },
        };

        manager
            .subscriptions
            .insert("reseller1".to_string(), subscription);

        let invoice = manager.calculate_invoice("reseller1").unwrap();

        // Base: $99.99, Overage: 2 nodes * $10 = $20
        assert!(invoice.total > 100.0);
        assert!(invoice.overage_charges > 0.0);
    }
}
