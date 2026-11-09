//! Routes and their OpenAPI description.
use super::dto::{CreateCrewRequest, JoinCrewRequest, MeDto, UpdateCrewRequest};
use super::handlers;
use crate::domain::CrewsService;
use axum::{Extension, Router, http::StatusCode};
use std::sync::Arc;
use toolkit::api::{OpenApiRegistry, OperationBuilder};

const TAG: &str = "Crews";

/// Registers the crews API on `router`.
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<CrewsService>,
) -> Router {
    router = OperationBuilder::get("/juggle-crews/v1/me")
        .operation_id("crews.me")
        .summary("You and your crew")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::me)
        .json_response_with_schema::<MeDto>(openapi, StatusCode::OK, "You and your crew")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::post("/juggle-crews/v1/crews")
        .operation_id("crews.create")
        .summary("Create a crew")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .json_request::<CreateCrewRequest>(openapi, "The crew and your name in it")
        .handler(handlers::create)
        .json_response_with_schema::<MeDto>(openapi, StatusCode::OK, "The new crew")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::post("/juggle-crews/v1/memberships")
        .operation_id("crews.join")
        .summary("Join a crew with its invite code")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .json_request::<JoinCrewRequest>(openapi, "The invite code and your name")
        .handler(handlers::join)
        .json_response_with_schema::<MeDto>(openapi, StatusCode::OK, "The crew you joined")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::post("/juggle-crews/v1/crew/invite-code")
        .operation_id("crews.rotate_invite")
        .summary("Replace the invite code (owner only)")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::rotate_invite)
        .json_response_with_schema::<MeDto>(openapi, StatusCode::OK, "A fresh invite code")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::put("/juggle-crews/v1/crew")
        .operation_id("crews.update")
        .summary("Rename the crew or change its time zone (owner only)")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .json_request::<UpdateCrewRequest>(openapi, "New name and zone")
        .handler(handlers::update)
        .json_response_with_schema::<MeDto>(openapi, StatusCode::OK, "The updated crew")
        .standard_errors(openapi)
        .register(router, openapi);
    router.layer(Extension(service))
}
