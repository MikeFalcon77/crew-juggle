//! Create-or-replace of app-wide secrets in the platform tenant.
use credstore_sdk::{
    CredStoreClientV1, CredStoreError, SecretRef, SecretValue, SharingMode, WritePrecondition,
};
use juggle_crews_sdk::secrets_provisioner_context;
use std::sync::Arc;
use toolkit::domain_model;

/// Writes secrets as the provisioner identity.
#[domain_model]
pub struct Provisioner {
    credstore: Arc<dyn CredStoreClientV1>,
}

impl Provisioner {
    /// A provisioner over credstore.
    pub fn new(credstore: Arc<dyn CredStoreClientV1>) -> Self {
        Self { credstore }
    }

    /// Creates the secret, or replaces it when it exists. Run on every start:
    /// the development credstore plugin keeps values in memory only.
    pub async fn put(&self, key: &SecretRef, value: &str) -> Result<(), CredStoreError> {
        let ctx = secrets_provisioner_context();
        let value = || SecretValue::new(value.as_bytes().to_vec());
        match self
            .credstore
            .create(&ctx, key, value(), SharingMode::Tenant)
            .await
        {
            Err(CredStoreError::Conflict) => {
                self.credstore
                    .put(
                        &ctx,
                        key,
                        value(),
                        SharingMode::Tenant,
                        WritePrecondition::Exists,
                    )
                    .await
            }
            other => other,
        }
    }
}
