// rr-ui/src/lib.rs

// Dioxus UI Module
#[cfg(any(feature = "server", feature = "web"))]
pub mod ui;

// Domain module (shared types available for both client and server)
pub mod domain;

// Server-only modules
#[cfg(feature = "server")]
pub mod api;
#[cfg(feature = "server")]
pub mod cli;
#[cfg(feature = "server")]
pub mod db;
#[cfg(feature = "server")]
pub mod jobs;
#[cfg(feature = "server")]
pub mod middleware;
pub mod models;
#[cfg(feature = "server")]
pub mod repositories;
#[cfg(feature = "server")]
pub mod rustray_client;
#[cfg(feature = "server")]
pub mod rustray_config;
#[cfg(feature = "server")]
pub mod rustray_process;
#[cfg(feature = "server")]
pub mod services;
#[cfg(feature = "server")]
pub mod transport;
#[cfg(feature = "server")]
pub mod tun_device;

// Hexagonal Architecture Modules (server-only)
#[cfg(feature = "server")]
pub mod adapters;
#[cfg(feature = "server")]
pub mod http_handlers;

// All server code is gated
#[cfg(feature = "server")]
mod server {
    use actix_cors::Cors;
    use actix_web::dev::Service;
    use actix_web::{App, HttpServer, middleware as actix_middleware, web};
    use dotenv::dotenv;
    use rustls::ServerConfig;
    use rustls_pemfile::{certs, pkcs8_private_keys};
    use std::fs::File;
    use std::io::BufReader;
    use std::sync::Arc;

    use crate::adapters::geo::MmapGeoRepository;
    use crate::adapters::{
        SurrealInboundRepository, SurrealOutboundRepository, SurrealSettingRepository,
        SurrealUserRepository,
    };
    use crate::api::system::SystemState;
    use crate::db::DbClient;
    use crate::models::AllSetting;
    use crate::repositories::SurrealRoutingRepository;
    use crate::repositories::balancer::SurrealBalancerRepository;
    use crate::repositories::setting::SettingOps;
    use crate::rustray_client::RustRayClient;
    use crate::rustray_process::SharedRustRayProcess;

    // AppState definition
    pub struct AppState {
        pub db: DbClient,
        pub user_repo: Arc<SurrealUserRepository>,
        pub inbound_repo: Arc<SurrealInboundRepository>,
        pub outbound_repo: Arc<SurrealOutboundRepository>,
        pub setting_repo: Arc<SurrealSettingRepository>,
        pub routing_repo: Arc<SurrealRoutingRepository>,
        pub balancer_repo: Arc<SurrealBalancerRepository>,
        pub geo_repo: Arc<MmapGeoRepository>,
        pub rustray: RustRayClient,
        pub tun: Option<crate::tun_device::SharedTunManager>,
        pub orchestrator: Arc<crate::services::orchestrator::Orchestrator>,
        pub shield: Arc<crate::services::shield::ShieldService>,
        pub audit: crate::services::audit::SharedAuditService,
        pub log_watcher: Arc<crate::services::log_watcher::LogWatcher>,
        pub mesh_orchestrator: crate::services::mesh::SharedMeshOrchestrator,
    }

    // TLS Configuration Loader
    fn load_rustls_config(cert_path: &str, key_path: &str) -> anyhow::Result<ServerConfig> {
        // Validate certificate file exists
        if !std::path::Path::new(cert_path).exists() {
            anyhow::bail!("Certificate file not found: {}", cert_path);
        }

        // Validate key file exists
        if !std::path::Path::new(key_path).exists() {
            anyhow::bail!("Private key file not found: {}", key_path);
        }

        log::info!("Loading TLS certificate from: {}", cert_path);
        log::info!("Loading TLS private key from: {}", key_path);

        // Load certificate chain
        let cert_file = File::open(cert_path)
            .map_err(|e| anyhow::anyhow!("Failed to open certificate file: {}", e))?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<rustls::pki_types::CertificateDer> = certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to parse certificate: {}", e))?;

        if cert_chain.is_empty() {
            anyhow::bail!("No certificates found in {}", cert_path);
        }

        // Load private key
        let key_file = File::open(key_path)
            .map_err(|e| anyhow::anyhow!("Failed to open private key file: {}", e))?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys = pkcs8_private_keys(&mut key_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        if keys.is_empty() {
            anyhow::bail!("No private keys found in {}", key_path);
        }

        let private_key = rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0));

        // Build TLS config with modern defaults (TLS 1.3)
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| anyhow::anyhow!("Failed to build TLS config: {}", e))?;

        log::info!("✓ TLS configuration loaded successfully");
        Ok(config)
    }

    pub async fn run_app(db_client: DbClient) -> anyhow::Result<()> {
        dotenv().ok();
        // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info")); // Handled in binary

        // let db_client = DbClient::init("rr-ui.db").await?; // Injected
        let settings = <AllSetting as SettingOps>::get(&db_client)
            .await?
            .unwrap_or_default();
        // Check PORT env var first (for dx serve / cloud deployment compatibility)
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(settings.web_port);

        // Extract camouflage settings
        let panel_secret_path = settings.panel_secret_path.clone();
        let decoy_site_path = settings.decoy_site_path.clone();

        // Use a fixed port for internal backend communication
        let backend_port = 10085;
        log::info!("Using internal backend port: {}", backend_port);

        // Initialize and start the Core Process FIRST so the client can connect to it
        log::info!("Initializing Core Process from settings...");
        let rustray_process =
            SharedRustRayProcess::init_from_db_with_port("config.json", &db_client, backend_port)
                .await;

        let mut rustray_client = RustRayClient::new(backend_port); // Core API port
        if let Err(e) = rustray_client.connect().await {
            log::warn!("RustRay Core offline: {}", e);
        }

        // Initialize Orchestrator first
        let orchestrator = Arc::new(crate::services::orchestrator::Orchestrator::new(
            backend_port,
        ));
        let orch_clone = orchestrator.clone();
        tokio::spawn(async move {
            orch_clone.connect().await;
        });

        let user_repo = Arc::new(SurrealUserRepository::new(db_client.clone()));
        let inbound_repo = Arc::new(SurrealInboundRepository::new(db_client.clone()));
        let outbound_repo = Arc::new(SurrealOutboundRepository::new(db_client.clone()));
        let setting_repo = Arc::new(SurrealSettingRepository::new(db_client.clone()));
        let routing_repo = Arc::new(SurrealRoutingRepository::new(db_client.clone()));
        let balancer_repo = Arc::new(SurrealBalancerRepository::new(db_client.clone()));

        // Initialize Geo Repository
        // Using default RustRay asset location or local
        let rustray_asset_path = std::env::var("RUSTRAY_ASSET_LOCATION")
            .unwrap_or_else(|_| "/usr/share/rustray".to_string());
        let geo_repo = Arc::new(MmapGeoRepository::new(
            std::path::PathBuf::from(&rustray_asset_path).join("geoip.dat"),
            std::path::PathBuf::from(&rustray_asset_path).join("geosite.dat"),
        ));

        // Initialize Log Watcher
        let log_watcher = Arc::new(crate::services::log_watcher::LogWatcher::new("access.log"));
        let lw_clone = log_watcher.clone();
        tokio::spawn(async move {
            lw_clone.run().await;
        });

        let app_state = Arc::new(AppState {
            db: db_client.clone(),
            user_repo,
            inbound_repo,
            outbound_repo,
            setting_repo,
            routing_repo,
            balancer_repo,
            geo_repo,
            rustray: rustray_client.clone(),
            tun: None, // TUN device managed separately via API
            orchestrator: orchestrator.clone(),
            shield: Arc::new(crate::services::shield::ShieldService::new()),
            audit: Arc::new(crate::services::audit::AuditService::new(Arc::new(
                tokio::sync::RwLock::new(db_client.clone()),
            ))),
            log_watcher,
            mesh_orchestrator: Arc::new(crate::services::mesh::MeshOrchestrator::new(Arc::new(
                tokio::sync::RwLock::new(db_client.clone()),
            ))),
        });
        let system_state = SystemState::new();

        // SharedRustRayProcess was initialized above to ensure connection before client start

        // Start Watchdog Job
        let wd_process = rustray_process.clone();
        let wd_db = db_client.clone();
        let wd_client = rustray_client.clone();
        tokio::spawn(async move {
            crate::jobs::watchdog::start_watchdog_job(wd_process, wd_db, wd_client).await;
        });

        // Orchestrator was initialized above to be included in AppState.
        // We already spawned the connection loop.
        let orchestrator = app_state.orchestrator.clone();

        // Start System Telemetry Service
        // let telemetry_service = crate::services::telemetry::TelemetryService::new();
        // telemetry_service.start().await;
        // Broadcast channel is now used primarily for Sniffer events
        let (tx, _rx) = tokio::sync::broadcast::channel::<actix_web::web::Bytes>(100);

        // Start State Reconciler
        // let reconciler = crate::services::reconciler::StateReconciler::new(
        //    db_client.clone(),
        //    orchestrator.clone(),
        // );
        // tokio::spawn(reconciler.run());

        // Start Sniffer Service
        // let sniffer = crate::services::sniffer::SniffingService::new(tx.clone());
        // tokio::spawn(sniffer.run());

        #[cfg(feature = "server")]
        {
            let job_state = app_state.clone();
            let job_tx = tx.clone();
            tokio::spawn(async move {
                crate::jobs::traffic::start_traffic_job(job_state, job_tx).await;
            });

            // let billing_state = app_state.clone();
            // tokio::spawn(async move {
            //    crate::jobs::billing::start_billing_job(billing_state).await;
            // });

            let bot_state = app_state.clone();
            tokio::spawn(async move {
                crate::services::tgbot::start_bot_loop(bot_state).await;
            });

            // Start UDS Manager
            let uds_manager = Arc::new(crate::adapters::uds_manager::UdsManager::new(
                db_client.clone(),
            ));
            tokio::spawn(async move {
                if let Err(e) = uds_manager.start_listener().await {
                    log::error!("UDS Manager failed: {}", e);
                }
            });
        }

        log::info!("Starting server on 0.0.0.0:{}", port);
        let server = HttpServer::new(move || {
            let app = App::new()
                .app_data(
                    actix_web::web::JsonConfig::default()
                        .limit(2 * 1024 * 1024)
                        .error_handler(|err, _req| {
                            actix_web::error::InternalError::from_response(
                                "",
                                actix_web::HttpResponse::BadRequest().finish(),
                            )
                            .into()
                        }),
                )
                .app_data(
                    actix_web::web::PathConfig::default().error_handler(|err, _req| {
                        actix_web::error::InternalError::from_response(
                            "",
                            actix_web::HttpResponse::BadRequest().finish(),
                        )
                        .into()
                    }),
                )
                // Ignore external scanner request parse errors to reduce log noise
                .app_data(actix_web::web::Data::new(SystemState::new()))
                .wrap(actix_web::middleware::Compress::default()) // Optional: compression
                // Header Scrubbing Middleware
                .wrap_fn(move |req, srv| {
                    let fut = srv.call(req);
                    async {
                        let mut res = fut.await?;
                        res.headers_mut().insert(
                            actix_web::http::header::SERVER,
                            actix_web::http::header::HeaderValue::from_static(
                                "nginx/1.24.0 (Ubuntu)",
                            ),
                        );
                        Ok(res)
                    }
                })
                // Decoy Middleware
                .wrap(crate::middleware::decoy::DecoyMiddleware::new(
                    panel_secret_path.clone(),
                    decoy_site_path.clone(),
                ))
                // Rate Limiting (100 req/min/IP)
                .wrap(crate::middleware::ratelimit::RateLimit::new(100, 60))
                // Audit Logging
                .wrap(crate::middleware::audit::AuditLog::new(
                    app_state.audit.clone(),
                ))
                .wrap(actix_middleware::Logger::default())
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header(),
                )
                .app_data(web::Data::from(app_state.clone()))
                .app_data(web::Data::new(system_state.clone()))
                .app_data(web::Data::new(rustray_process.clone()))
                .app_data(web::Data::new(tx.clone()))
                .configure(crate::api::config);

            // Dioxus Server Functions - only available with web feature
            #[cfg(feature = "web")]
            let app = app.service(web::resource("/api/{name}").route(web::post().to(
                |req: actix_web::HttpRequest, payload: web::Payload| async move {
                    server_fn::actix::handle_server_fn(req, payload).await
                },
            )));

            // Static file serving (SPA support)
            app.default_service(web::to(
                crate::http_handlers::static_files::serve_static_asset,
            ))
        });

        // Check if TLS is configured and bind accordingly
        let bound_server = if let (Some(cert_path), Some(key_path)) =
            (&settings.web_cert_file, &settings.web_key_file)
        {
            log::info!("🔒 Starting HTTPS server on 0.0.0.0:{}", port);

            // Load TLS configuration with fail-fast validation
            let tls_config = load_rustls_config(cert_path, key_path).map_err(|e| {
                log::error!("❌ CRITICAL: TLS configuration failed: {}", e);
                log::error!("❌ Server will NOT start without valid TLS certificates");
                log::error!("❌ Please check certificate paths in database settings");
                e
            })?;

            server.bind_rustls_0_23(("0.0.0.0", port), tls_config)?
        } else {
            log::warn!("⚠️  WARNING: No TLS certificates configured!");
            log::warn!("⚠️  Server will start in HTTP mode (insecure)");
            log::warn!("⚠️  Use 'rr-ui cert --cert <path> --key <path>' to configure HTTPS");
            server.bind(("0.0.0.0", port))?
        };

        // Graceful shutdown signal handler
        let server_handle = bound_server.run();
        let server_task = server_handle.handle();

        tokio::spawn(async move {
            // Wait for shutdown signal
            #[cfg(unix)]
            {
                use tokio::signal::unix::{SignalKind, signal};
                let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler failed");
                let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler failed");

                tokio::select! {
                    _ = sigterm.recv() => {
                        log::info!("📛 Received SIGTERM, initiating graceful shutdown...");
                    }
                    _ = sigint.recv() => {
                        log::info!("📛 Received SIGINT (Ctrl+C), initiating graceful shutdown...");
                    }
                }
            }

            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Ctrl+C handler failed");
                log::info!("📛 Received Ctrl+C, initiating graceful shutdown...");
            }

            // Trigger graceful shutdown
            let _ = server_task.stop(true);
        });

        server_handle.await?;
        log::info!("✅ Server shutdown complete");
        Ok(())
    }
} // End of server module

// Re-export server types and functions for backward compatibility
#[cfg(feature = "server")]
pub use server::{AppState, run_app};
