//! The secrets gear.
use crate::config::SecretsConfig;
use crate::domain::Provisioner;
use async_trait::async_trait;
use credstore as _;
use credstore_sdk::{CredStoreClientV1, SecretRef};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use toolkit::{Gear, GearCtx};

/// The secrets to write, already checked: credstore reference and value.
type Secrets = Vec<(SecretRef, String)>;

/// Writes the configured secrets into credstore once the platform is up.
#[toolkit::gear(name = "juggle-secrets", deps = [credstore], capabilities = [stateful], lifecycle(entry = "serve", stop_timeout = "5s"))]
#[derive(Default)]
pub struct SecretsGear {
    state: OnceLock<(Arc<Provisioner>, Secrets)>,
}

#[async_trait]
impl Gear for SecretsGear {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        let config: SecretsConfig = ctx.config_expanded_or_default()?;
        let mut secrets = Vec::new();
        for entry in config.secrets {
            let key = SecretRef::new(&entry.key)
                .map_err(|e| anyhow::anyhow!("juggle-secrets: invalid key {}: {e}", entry.key))?;
            let value = entry.value.trim().to_owned();
            if !value.is_empty() {
                secrets.push((key, value));
            }
        }
        let credstore = ctx.client_hub().get::<dyn CredStoreClientV1>()?;
        self.state
            .set((Arc::new(Provisioner::new(credstore)), secrets))
            .map_err(|_| anyhow::anyhow!("secrets gear already initialized"))?;
        Ok(())
    }
}

impl SecretsGear {
    /// Retries each secret with backoff until it is written: credstore's
    /// plugin may become reachable a moment after the server starts.
    async fn serve(self: Arc<Self>, cancel: CancellationToken) -> anyhow::Result<()> {
        let (provisioner, secrets) = self
            .state
            .get()
            .ok_or_else(|| anyhow::anyhow!("secrets gear not initialized"))?;
        for (key, value) in secrets {
            let mut delay = Duration::from_millis(250);
            loop {
                match provisioner.put(key, value).await {
                    Ok(()) => {
                        tracing::info!(key = %key.as_ref(), "secret provisioned");
                        break;
                    }
                    Err(error) => {
                        tracing::warn!(key = %key.as_ref(), %error, "secret not provisioned yet; retrying");
                    }
                }
                tokio::select! {
                    () = cancel.cancelled() => return Ok(()),
                    () = tokio::time::sleep(delay) => {}
                }
                delay = (delay * 2).min(Duration::from_secs(30));
            }
        }
        cancel.cancelled().await;
        Ok(())
    }
}
