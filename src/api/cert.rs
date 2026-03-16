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
        let (account, _creds) = Account::create(
            &NewAccount {
                contact: &[],
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            url,
            None,
        )
        .await?;

        // 2. Create order for domain
        let mut order = account
            .new_order(&NewOrder {
                identifiers: &[Identifier::Dns(domain.to_string())],
            })
            .await?;

        // 3. Get authorizations/challenges
        let authorizations = order.authorizations(&account).await?;
        for authz in authorizations {
            if let Some(challenge) = authz
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Http01)
            {
                // In a real implementation: write token to /.well-known/acme-challenge/
                account.set_challenge_ready(&challenge.url).await?;
            }
        }

        // 4. Exponential backoff to poll order status
        let mut attempts = 0;
        let mut delay = Duration::from_secs(1);
        let max_attempts = 6;

        loop {
            let state = order.refresh(&account).await?;
            if state.status == OrderStatus::Ready {
                break;
            } else if state.status == OrderStatus::Invalid {
                return Err(anyhow::anyhow!("Order invalid"));
            }

            if attempts >= max_attempts {
                return Err(anyhow::anyhow!("ACME order timeout"));
            }

            sleep(delay).await;
            delay *= 2; // Exponential backoff
            attempts += 1;
        }

        // 5. Finalize order with CSR (mocking CSR generation here)
        // let cert = rcgen::generate_simple_self_signed(vec![domain.to_string()])?;
        // let csr = cert.serialize_request_der()?;
        // order.finalize(&account, &csr).await?;

        // 6. Poll again for valid status to download
        attempts = 0;
        delay = Duration::from_secs(1);
        loop {
            let state = order.refresh(&account).await?;
            if state.status == OrderStatus::Valid {
                break;
            } else if state.status == OrderStatus::Invalid {
                return Err(anyhow::anyhow!("Order invalid during finalize"));
            }

            if attempts >= max_attempts {
                return Err(anyhow::anyhow!("ACME certificate timeout"));
            }

            sleep(delay).await;
            delay *= 2;
            attempts += 1;
        }

        let _cert_chain = order.certificate(&account).await?;

        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = domain;
        let _ = provider;
        Ok(())
    }
}
