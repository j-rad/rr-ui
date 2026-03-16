// src/api/inbound.rs
use crate::AppState;
use crate::domain::ports::{InboundRepository, OutboundRepository};
use crate::models::{GeneralResponse, Inbound};
use actix_web::{HttpResponse, Responder, get, post, web};
#[cfg(feature = "server")]
use std::str::FromStr;
#[cfg(feature = "server")]
use surrealdb::sql::Thing;

#[get("/list")]
pub async fn list(data: web::Data<AppState>) -> impl Responder {
    let result = data.inbound_repo.find_all().await;
    match result {
        Ok(inbounds) => HttpResponse::Ok().json(GeneralResponse::success(
            "success",
            Some(serde_json::to_value(inbounds).unwrap()),
        )),
        Err(e) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error(&format!("Repo Error: {}", e))),
    }
}

#[post("/add")]
pub async fn add(
    data: web::Data<AppState>,
    inbound: web::Json<Inbound<'static>>,
) -> impl Responder {
    #[cfg(feature = "server")]
    {
        use crate::rustray_process::validate_config;

        let new_inbound = inbound.into_inner();

        // 1. Fetch current models
        // 1. Fetch current models
        let mut inbounds = match data.inbound_repo.find_all().await {
            Ok(models) => models,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(GeneralResponse::error(&format!("Repo Error: {}", e)));
            }
        };
        let outbounds = match data.outbound_repo.find_all().await {
            Ok(models) => models,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(GeneralResponse::error(&format!("Repo Error: {}", e)));
            }
        };

        // 2. Add new inbound to the list (simulate)
        inbounds.push(new_inbound.clone());

        // 3. Build full config
        let temp_config =
            crate::rustray_config::RustRayConfigBuilder::build_from_models(&inbounds, &outbounds);

        // 4. Validate
        if let Err(e) = validate_config(&temp_config, &data.db).await {
            return HttpResponse::BadRequest().json(GeneralResponse::error(&format!(
                "Configuration validation failed: {}",
                e
            )));
        }

        // 5. Save if valid
        // 5. Save if valid
        let result = data.inbound_repo.create(new_inbound).await;
        match result {
            Ok(created) => HttpResponse::Ok().json(GeneralResponse::success(
                "Inbound added",
                Some(serde_json::to_value(created).unwrap()),
            )),
            Err(e) => HttpResponse::InternalServerError()
                .json(GeneralResponse::error(&format!("Repo Error: {}", e))),
        }
    }
    #[cfg(not(feature = "server"))]
    {
        HttpResponse::NotImplemented().finish()
    }
}

#[post("/update/{id}")]
pub async fn update(
    data: web::Data<AppState>,
    path: web::Path<String>,
    inbound: web::Json<Inbound<'static>>,
) -> impl Responder {
    let id_str = path.into_inner();
    let mut new_inbound = inbound.into_inner();

    #[cfg(feature = "server")]
    {
        use crate::rustray_process::validate_config;

        // Try to parse ID
        let thing_id = if id_str.contains(":") {
            Thing::from_str(&id_str).ok()
        } else {
            Thing::from_str(&format!("inbound:{}", id_str)).ok()
        };
        new_inbound.id = thing_id.clone();

        if let Some(tid) = thing_id {
            // 1. Fetch current models
            let mut inbounds = match data.inbound_repo.find_all().await {
                Ok(models) => models,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(GeneralResponse::error(&format!("Repo Error: {}", e)));
                }
            };
            let outbounds = match data.outbound_repo.find_all().await {
                Ok(models) => models,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(GeneralResponse::error(&format!("Repo Error: {}", e)));
                }
            };

            // 2. Locate and replace the inbound
            let mut found = false;
            for inbound_model in &mut inbounds {
                if let Some(ref id) = inbound_model.id {
                    if id == &tid {
                        *inbound_model = new_inbound.clone();
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                return HttpResponse::NotFound()
                    .json(GeneralResponse::error("Inbound not found for validation"));
            }

            // 3. Build full config
            let temp_config = crate::rustray_config::RustRayConfigBuilder::build_from_models(
                &inbounds, &outbounds,
            );

            // 4. Validate
            if let Err(e) = validate_config(&temp_config, &data.db).await {
                return HttpResponse::BadRequest().json(GeneralResponse::error(&format!(
                    "Configuration validation failed: {}",
                    e
                )));
            }

            // 5. Update DB
            // 5. Update DB
            let result = data
                .inbound_repo
                .update(&tid.id.to_string(), new_inbound)
                .await;

            match result {
                Ok(updated) => HttpResponse::Ok().json(GeneralResponse::success(
                    "Inbound updated",
                    Some(serde_json::to_value(updated).unwrap()),
                )),
                Err(e) => HttpResponse::InternalServerError()
                    .json(GeneralResponse::error(&format!("Repo Error: {}", e))),
            }
        } else {
            HttpResponse::BadRequest().json(GeneralResponse::error("Invalid ID format"))
        }
    }
    #[cfg(not(feature = "server"))]
    {
        HttpResponse::NotImplemented().finish()
    }
}

#[post("/del/{id}")]
pub async fn del(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id_str = path.into_inner();

    #[cfg(feature = "server")]
    {
        let thing_id = if id_str.contains(":") {
            Thing::from_str(&id_str).ok()
        } else {
            Thing::from_str(&format!("inbound:{}", id_str)).ok()
        };

        if let Some(tid) = thing_id {
            let result = data.inbound_repo.delete(&tid.id.to_string()).await;
            match result {
                Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Inbound deleted", None)),
                Err(e) => HttpResponse::InternalServerError()
                    .json(GeneralResponse::error(&format!("Repo Error: {}", e))),
            }
        } else {
            HttpResponse::BadRequest().json(GeneralResponse::error("Invalid ID format"))
        }
    }
    #[cfg(not(feature = "server"))]
    {
        HttpResponse::NotImplemented().finish()
    }
}
