//! Routes of the routines API and their OpenAPI description.
use super::dto;
use super::handlers::{self, App};
use axum::{Extension, Router, http::StatusCode};
use std::sync::Arc;
use toolkit::api::{OpenApiRegistry, operation_builder::OperationBuilder};

const TAG: &str = "Routines";

/// Registers the routines API on `router`.
pub fn register_routes(mut router: Router, openapi: &dyn OpenApiRegistry, app: Arc<App>) -> Router {
    macro_rules! read {
        ($path:expr, $id:expr, $summary:expr, $handler:path, $out:ty) => {
            router = OperationBuilder::get($path)
                .operation_id($id)
                .summary($summary)
                .tag(TAG)
                .authenticated()
                .no_license_required()
                .handler($handler)
                .json_response_with_schema::<$out>(openapi, StatusCode::OK, $summary)
                .standard_errors(openapi)
                .register(router, openapi);
        };
    }
    macro_rules! write {
        ($method:ident, $path:expr, $id:expr, $summary:expr, $handler:path, $in:ty, $out:ty) => {
            router = OperationBuilder::$method($path)
                .operation_id($id)
                .summary($summary)
                .tag(TAG)
                .authenticated()
                .no_license_required()
                .json_request::<$in>(openapi, $summary)
                .handler($handler)
                .json_response_with_schema::<$out>(openapi, StatusCode::OK, $summary)
                .standard_errors(openapi)
                .register(router, openapi);
        };
    }
    read!(
        "/juggle-routines/v1/me",
        "routines.me",
        "Your account, crew zone and today",
        handlers::me,
        dto::MeView
    );
    read!(
        "/juggle-routines/v1/me/preferences",
        "routines.preferences",
        "Display zone of you and your crewmates",
        handlers::preferences,
        dto::PreferencesView
    );
    write!(
        put,
        "/juggle-routines/v1/me/preferences",
        "routines.edit_preferences",
        "Change your display zone",
        handlers::edit_preferences,
        dto::PreferencesEdit,
        dto::Preferences
    );
    read!(
        "/juggle-routines/v1/day",
        "routines.day",
        "One day of the crew's routines",
        handlers::day,
        dto::DayView
    );
    write!(
        put,
        "/juggle-routines/v1/occurrences/{date}/{id}",
        "routines.mark",
        "Check off or undo one task",
        handlers::mark,
        dto::Mark,
        dto::Occurrence
    );
    read!(
        "/juggle-routines/v1/schedule",
        "routines.schedule",
        "Every routine and who does it",
        handlers::schedule,
        dto::ScheduleView
    );
    write!(
        put,
        "/juggle-routines/v1/schedule",
        "routines.edit",
        "Add or change a routine",
        handlers::edit,
        dto::Edit,
        dto::Version
    );
    write!(
        put,
        "/juggle-routines/v1/schedule/{id}/appearance",
        "routines.appearance",
        "Change a routine's mood",
        handlers::appearance,
        dto::AppearanceEdit,
        dto::Appearance
    );
    read!(
        "/juggle-routines/v1/people",
        "routines.people",
        "People of the crew",
        handlers::people,
        dto::PeopleView
    );
    write!(
        put,
        "/juggle-routines/v1/people",
        "routines.edit_person",
        "Add or change a person",
        handlers::edit_person,
        dto::PersonEdit,
        dto::Person
    );
    write!(
        delete,
        "/juggle-routines/v1/people/{id}/avatar",
        "routines.remove_avatar",
        "Remove a photo",
        handlers::remove_avatar,
        dto::AvatarEdit,
        dto::AvatarMeta
    );
    write!(
        post,
        "/juggle-routines/v1/push/subscriptions",
        "routines.subscribe",
        "Subscribe this browser to reminders",
        handlers::subscribe,
        dto::Subscribe,
        dto::SubscriptionCreated
    );
    write!(
        post,
        "/juggle-routines/v1/push/test",
        "routines.test_push",
        "Send a test reminder",
        handlers::test_push,
        dto::TestPush,
        dto::Done
    );
    router = OperationBuilder::get("/juggle-routines/v1/people/{id}/avatar")
        .operation_id("routines.avatar")
        .summary("A person's photo")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::avatar)
        .text_response(StatusCode::OK, "PNG photo", "image/png")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::put("/juggle-routines/v1/people/{id}/avatar")
        .operation_id("routines.upload_avatar")
        .summary("Upload a photo (multipart: operation_id, expected_revision, file)")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::upload_avatar)
        .json_response_with_schema::<dto::AvatarMeta>(openapi, StatusCode::OK, "The new photo")
        .standard_errors(openapi)
        .register(router, openapi);
    router = OperationBuilder::delete("/juggle-routines/v1/push/subscriptions/{id}")
        .operation_id("routines.unsubscribe")
        .summary("Stop reminders in this browser")
        .tag(TAG)
        .authenticated()
        .no_license_required()
        .handler(handlers::unsubscribe)
        .json_response_with_schema::<dto::Done>(openapi, StatusCode::OK, "Unsubscribed")
        .standard_errors(openapi)
        .register(router, openapi);
    router
        .layer(axum::extract::DefaultBodyLimit::max(768 * 1024))
        .layer(Extension(app))
}
