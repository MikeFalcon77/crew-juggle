//! The two anonymous sign-in routes. They keep the paths the web app has
//! always used, under `/juggle-crews/v1`.
use super::dto::{DemoSessionRequest, DemoTokenDto, SessionConfigDto};
use super::handlers::{self, SignInState};
use axum::{Extension, Router, http::StatusCode};
use toolkit::api::{OpenApiRegistry, OperationBuilder};

const TAG: &str = "Sign-in";

/// Registers the sign-in routes on `router`.
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    state: SignInState,
) -> Router {
    router = OperationBuilder::get("/juggle-crews/v1/config")
        .operation_id("crews.config")
        .summary("How this server signs people in")
        .tag(TAG)
        .anonymous()
        .handler(handlers::config)
        .json_response_with_schema::<SessionConfigDto>(openapi, StatusCode::OK, "How to sign in")
        .register(router, openapi);
    router = OperationBuilder::post("/juggle-crews/v1/demo/sessions")
        .operation_id("crews.demo_session")
        .summary("Sign in as a demo character (demo mode only)")
        .tag(TAG)
        .anonymous()
        .json_request::<DemoSessionRequest>(openapi, "The character")
        .handler(handlers::demo_session)
        .json_response_with_schema::<DemoTokenDto>(openapi, StatusCode::OK, "Demo sign-in token")
        .standard_errors(openapi)
        .register(router, openapi);
    router.layer(Extension(state))
}
