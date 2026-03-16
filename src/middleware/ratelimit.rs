// src/middleware/ratelimit.rs
//! Middleware for API rate limiting
//!
//! Implements a simple sliding window or fixed window rate limiter
//! independent of external crates like governor to simplify dependencies.

use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Simple in-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    // Map IP -> (Request Count, Window Start)
    limits: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, ip: &str) -> bool {
        let mut limits = self.limits.lock().unwrap();
        let now = Instant::now();

        // Remove expired entries (rudimentary cleanup)
        // In prod, use a dedicated cleanup task or TTL cache
        if limits.len() > 10000 {
            limits.retain(|_, (_, start)| now.duration_since(*start) < self.window_duration);
        }

        let entry = limits.entry(ip.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) > self.window_duration {
            // Window expired, reset
            entry.0 = 1;
            entry.1 = now;
            true
        } else {
            // Window active
            if entry.0 < self.max_requests {
                entry.0 += 1;
                true
            } else {
                false
            }
        }
    }
}

/// Rate Limit Middleware
pub struct RateLimit {
    limiter: RateLimiter,
}

impl RateLimit {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            limiter: RateLimiter::new(max_requests, window_secs),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    limiter: RateLimiter,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let limiter = self.limiter.clone();

        if let Some(addr) = req.peer_addr() {
            let ip = addr.ip().to_string();
            if !limiter.check(&ip) {
                return Box::pin(async {
                    Err(actix_web::error::ErrorTooManyRequests(
                        "Rate limit exceeded",
                    ))
                });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
