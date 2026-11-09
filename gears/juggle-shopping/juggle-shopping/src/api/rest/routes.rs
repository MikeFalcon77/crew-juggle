//! Routes of the shopping API and their OpenAPI description.
use super::dto::{
    CommandApplied, CommandRequest, LedgerPage, ReceiptPhotos, StateView, UploadAccepted,
};
use super::handlers;
use crate::domain::Service;
use axum::{Extension, Router, extract::DefaultBodyLimit, http::StatusCode};
use std::sync::Arc;
use toolkit::api::{OpenApiRegistry, OperationBuilder};

const TAG: &str = "Shopping";

/// Registers the shopping API on `router`.
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    router = OperationBuilder::get("/juggle-shopping/v1/state")
        .operation_id("shopping.state")
        .summary("The list, catalog, receipts and purchases")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::state)
        .json_response_with_schema::<StateView>(openapi, StatusCode::OK, "Crew shopping state")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::post("/juggle-shopping/v1/commands")
        .operation_id("shopping.command")
        .summary("Change the list, a product, a rule or a receipt")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .json_request::<CommandRequest>(openapi, "The command")
        .handler(handlers::command)
        .json_response_with_schema::<CommandApplied>(openapi, StatusCode::OK, "Applied")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::post("/juggle-shopping/v1/receipts")
        .operation_id("shopping.upload")
        .summary("Upload receipt photos (multipart: operation_id, photo × 1–5)")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::upload)
        .json_response_with_schema::<UploadAccepted>(
            openapi,
            StatusCode::ACCEPTED,
            "Receipt queued",
        )
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::get("/juggle-shopping/v1/ledger")
        .operation_id("shopping.ledger")
        .summary("Purchase events, 200 per page")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::ledger)
        .json_response_with_schema::<LedgerPage>(openapi, StatusCode::OK, "Ledger page")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::get("/juggle-shopping/v1/receipts/{id}/photos")
        .operation_id("shopping.receipt_photos")
        .summary("A receipt's photos while they are kept")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::receipt_photos)
        .json_response_with_schema::<ReceiptPhotos>(
            openapi,
            StatusCode::OK,
            "Private source photos",
        )
        .standard_errors(openapi)
        .register(router, openapi);
    router
        .layer(DefaultBodyLimit::max(51 * 1024 * 1024))
        .layer(Extension(service))
}
