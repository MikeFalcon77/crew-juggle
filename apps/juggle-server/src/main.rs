use clap::Parser;
use std::path::PathBuf;
use toolkit::bootstrap::{AppConfig, run_migrate, run_server};
// Every gear the server runs: the platform's own, then crew-juggle's.
use {
    api_gateway as _, authn_resolver as _, authz_resolver as _, credstore as _, oagw as _,
    single_tenant_tr_plugin as _, static_credstore_plugin as _, tenant_resolver as _,
    types_registry as _,
};
use {
    juggle_crews as _, juggle_crews_authn_plugin as _, juggle_crews_authz_plugin as _,
    juggle_routines as _, juggle_secrets as _, juggle_shopping as _,
};

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "config/production.yaml")]
    config: PathBuf,
    /// Apply migrations and exit.
    #[arg(long)]
    migrate_only: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Local secrets remain in ignored .env; deployment environment wins.
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    // Ports and everything else come from the YAML file; any value can be
    // overridden with APP__GEARS__<GEAR>__CONFIG__<KEY> environment variables.
    let config = AppConfig::load_or_default(Some(&cli.config))?;
    if cli.migrate_only {
        run_migrate(config).await
    } else {
        run_server(config).await
    }
}
