// rr-ui/src/api/mod.rs
use actix_web::web;

pub mod auth;
#[cfg(feature = "server")]
pub mod backup;
#[cfg(feature = "server")]
pub mod balancer;
#[cfg(feature = "server")]
pub mod cert;
pub mod client;
#[cfg(feature = "server")]
pub mod geo;
#[cfg(feature = "server")]
pub mod inbound;
#[cfg(feature = "server")]
pub mod log;
#[cfg(feature = "server")]
pub mod routing;
pub mod rustray_control;
#[cfg(feature = "server")]
pub mod server;
pub mod setting;
#[cfg(feature = "server")]
pub mod signaling;
#[cfg(feature = "server")]
pub mod sub;
pub mod system;
pub mod telemetry;
#[cfg(feature = "server")]
pub mod tgbot;
pub mod tun;
#[cfg(feature = "server")]
pub mod warp;

use crate::middleware::auth::Auth;

pub fn config(cfg: &mut web::ServiceConfig) {
    let mut panel_scope = web::scope("/panel/api")
        .wrap(Auth)
        .service(
            web::scope("/server")
                .service(system::get_status)
                .service(system::get_mode)
                .service(rustray_control::status)
                .service(rustray_control::restart)
                .service(rustray_control::stop)
                .service(rustray_control::get_rustray_version)
                .service(rustray_control::install_rustray)
                .service(system::get_banned_ips)
                .service(system::unban_ip)
                .service(system::clear_banned_ips)
                .configure(server::config),
        )
        .service(
            web::scope("/setting")
                .service(setting::get_all_settings)
                .service(setting::update_setting),
        )
        .service(
            web::scope("/client")
                .service(client::get_traffic)
                .service(client::reset_traffic),
        )
        .service(
            web::scope("/tun")
                .service(tun::get_status)
                .service(tun::start)
                .service(tun::stop)
                .service(tun::update_config),
        );

    #[cfg(feature = "server")]
    {
        panel_scope = panel_scope
            .service(
                web::scope("/inbounds")
                    .service(inbound::list)
                    .service(inbound::add)
                    .service(inbound::update)
                    .service(inbound::del),
            )
            .service(
                web::scope("/warp")
                    .service(warp::status)
                    .service(warp::register),
            )
            .service(web::scope("/cert").service(cert::issue_cert))
            .service(
                web::scope("/backup")
                    .service(backup::export_db)
                    .service(backup::import_db),
            )
            .service(
                web::scope("/routing")
                    .service(routing::list_rules)
                    .service(routing::add_rule)
                    .service(routing::del_rule),
            )
            .service(
                web::scope("/balancer")
                    .service(balancer::list)
                    .service(balancer::add)
                    .service(balancer::del),
            )
            .service(
                web::scope("/tgbot")
                    .service(tgbot::get_config)
                    .service(tgbot::update_config)
                    .service(tgbot::test_bot),
            )
            .configure(geo::geo_api_config)
            .service(web::scope("/signaling").configure(signaling::config));
    }

    let mut root_scope = web::scope("")
        .service(auth::login)
        .service(auth::verify_mfa)
        .service(auth::get_mfa_secret)
        .service(panel_scope);

    #[cfg(feature = "server")]
    {
        root_scope = root_scope.service(sub::get_subscription);
    }

    // Telemetry WebSocket
    root_scope = root_scope.service(telemetry::ws_stats);

    cfg.service(root_scope);
}
