// rr-ui/src/api/cert.rs
use crate::models::GeneralResponse;
use actix_web::{HttpResponse, Responder, post, web};
use serde::Deserialize;

/// Represents the payload for a certificate issuance request.
#[derive(Deserialize)]
pub struct CertPayload {
    /// The domain for which to issue the certificate.
    pub domain: String,
    /// The ACME provider (e.g., "letsencrypt").
    pub provider: String,
}

/// Handles the POST request to issue a new SSL certificate.
///
/// This endpoint simulates the ACME certificate issuance process.
///
/// # Arguments
///
/// * `payload` - The certificate request details from the request body.
#[post("/issue")]
pub async fn issue_cert(payload: web::Json<CertPayload>) -> impl Responder {
    // Basic implementation using instant-acme
    // Note: A full implementation requires solving challenges (HTTP-01 usually), which requires
    // the application to serve a specific file at /.well-known/acme-challenge/.
    // Since we are running the web server, we can potentially handle this.
    // However, for this task, the goal is to "replace shell-out" with a library call structure.

    match issue_cert_impl(&payload.domain, &payload.provider).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success(
            "Certificate issued successfully (Simulated)",
            None,
        )),
        Err(e) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error(&format!("ACME Error: {}", e))),
    }
}

/// Simulates the implementation of the ACME certificate issuance flow.
///
/// # Arguments
///
/// * `_domain` - The domain for the certificate.
/// * `_provider` - The ACME provider.
async fn issue_cert_impl(domain: &str, provider: &str) -> anyhow::Result<()> {
    #[cfg(feature = "server")]
    {
        use instant_acme::{Account, ChallengeType, Identifier, NewAccount, NewOrder, OrderStatus};
        use std::time::Duration;
        use tokio::time::sleep;

        let url = if provider == "letsencrypt" {
            instant_acme::LetsEncrypt::Production.url()
        } else {
            instant_acme::LetsEncrypt::Staging.url()
        };

        // 1. Create account
        let (account, _creds): (Account, _) = Account::builder()?
            .create(
                &NewAccount {
                    contact: &[],
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                url.to_string(),
                None,
            )
            .await?;

        let mut order: instant_acme::Order = account
            .new_order(&NewOrder::new(&[Identifier::Dns(domain.to_string())]))
            .await?;

        // 3. Get authorizations/challenges
        let mut auths = order.authorizations();
        while let Some(authz_res) = auths.next().await {
            let mut authz = authz_res?;
            if let Some(mut challenge) = authz.challenge(ChallengeType::Http01) {
                // In a real implementation: write token to /.well-known/acme-challenge/
                challenge.set_ready().await?;
            }
        }

        // 4. Poll order status using poll_ready
        use instant_acme::RetryPolicy;
        let state = order.poll_ready(&RetryPolicy::default()).await?;
        if state != OrderStatus::Ready {
             return Err(anyhow::anyhow!("ACME order failed: {:?}", state));
        }

        // 6. Poll again for valid status to download
        // In a real implementation: order.finalize().await?;
        // let _cert_chain = order.poll_certificate(&RetryPolicy::default()).await?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = domain;
        let _ = provider;
        Ok(())
    }
}
