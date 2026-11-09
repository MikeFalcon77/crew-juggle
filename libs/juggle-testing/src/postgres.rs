//! A PostgreSQL for database tests, from Docker or from `JUGGLE_TEST_PG_URL`.
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres as PostgresImage;

/// Postgres image tag for database tests; `JUGGLE_PG_TAG` overrides it.
pub const POSTGRES_TAG: &str = "18-alpine";

/// A database the tests may create schemas in. Dropping it stops the container.
pub struct Postgres {
    /// Connection URL.
    pub url: String,
    _container: Option<ContainerAsync<PostgresImage>>,
}

fn require_docker() -> bool {
    std::env::var("JUGGLE_PG_REQUIRE_DOCKER").is_ok_and(|v| !v.is_empty() && v != "0")
}

/// A Postgres for one test, or `None` when Docker is unavailable and
/// `JUGGLE_PG_REQUIRE_DOCKER` is not set (then the test skips itself).
///
/// # Panics
///
/// When the container fails to start and `JUGGLE_PG_REQUIRE_DOCKER=1`, as in CI.
pub async fn postgres() -> Option<Postgres> {
    if let Ok(url) = std::env::var("JUGGLE_TEST_PG_URL") {
        return Some(Postgres {
            url,
            _container: None,
        });
    }
    let tag = std::env::var("JUGGLE_PG_TAG").unwrap_or_else(|_| POSTGRES_TAG.to_owned());
    let started = PostgresImage::default()
        .with_user("juggle")
        .with_password("juggle")
        .with_db_name("juggle")
        .with_tag(tag)
        .start()
        .await;
    let container = match started {
        Ok(container) => container,
        Err(error) => {
            assert!(
                !require_docker(),
                "Postgres for the database tests did not start: {error}"
            );
            tracing::warn!(%error, "no Docker: skipping a database test");
            return None;
        }
    };
    let port = container.get_host_port_ipv4(5432).await.ok()?;
    Some(Postgres {
        url: format!("postgres://juggle:juggle@127.0.0.1:{port}/juggle"),
        _container: Some(container),
    })
}
