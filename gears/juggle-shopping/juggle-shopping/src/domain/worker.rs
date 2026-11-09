//! The background worker: scans queued receipts, imports pasted lists and
//! classifies new products, one crew at a time.
use super::error::{Error, Result};
use super::service::Service;
use chrono::Utc;
use juggle_crews_sdk::worker_context;
use toolkit_security::SecurityContext;
use uuid::Uuid;

impl Service {
    /// One worker pass over every crew. A failing crew does not stop the others.
    pub async fn tick(&self) -> Result<()> {
        let crews = self
            .crews
            .crew_ids(&worker_context(Uuid::nil()))
            .await
            .map_err(|_| Error::Storage)?;
        for crew in crews {
            if let Err(error) = self.tick_crew(worker_context(crew)).await {
                tracing::warn!(%crew, %error, "shopping worker failed for one crew");
            }
        }
        Ok(())
    }
    async fn tick_crew(&self, ctx: SecurityContext) -> Result<()> {
        let scope = self.scope(&ctx).await?;
        self.store.ensure(&scope).await?;
        self.store.expire(&scope, Utc::now()).await?;
        if !self.model.configured() {
            return Ok(());
        }
        let lease = Uuid::new_v4().to_string();
        let now = Utc::now();
        let claimed = self
            .mutate(&ctx, &[], |state| {
                let catalog = state.catalog.clone();
                let rules = state.rules.clone();
                if let Some(receipt) = state.receipts.iter_mut().find(|r| {
                    ((r.status == "queued" && r.next_attempt_at.is_none_or(|at| at <= now))
                        || (r.status == "processing" && r.lease_until.is_none_or(|at| at < now)))
                        && r.photo_expires_at > now
                }) {
                    receipt.status = "processing".into();
                    receipt.lease = Some(lease.clone());
                    receipt.lease_until = Some(now + chrono::Duration::minutes(3));
                    receipt.attempts += 1;
                    receipt.revision += 1;
                    return Ok(Some((receipt.clone(), catalog, rules)));
                }
                Ok(None)
            })
            .await?;
        if let Some((receipt, catalog, rules)) = claimed {
            let zone = self
                .crews
                .crew(&ctx)
                .await
                .ok()
                .and_then(|crew| crew.timezone.parse().ok())
                .unwrap_or(chrono_tz::UTC);
            let result = match self.store.photos(&scope, &receipt.id, now).await {
                Ok(photos) if !photos.is_empty() => {
                    self.model.receipt(&photos, &catalog, &rules).await
                }
                _ => Err(Error::Model(
                    "The receipt photo is gone. Add it again.".into(),
                )),
            };
            self.mutate(&ctx, &[], |state| {
                let Some(index) = state
                    .receipts
                    .iter()
                    .position(|r| r.id == receipt.id && r.lease.as_deref() == Some(&lease))
                else {
                    return Ok(());
                };
                match &result {
                    Ok(extraction) => {
                        let before = state.clone();
                        if let Err(error) = state.finish_receipt(index, extraction.clone(), zone) {
                            *state = before;
                            let r = &mut state.receipts[index];
                            r.status = "failed".into();
                            r.issues = vec![error.to_string()];
                            r.lease = None;
                            r.lease_until = None;
                            r.revision += 1;
                        }
                    }
                    Err(error) => {
                        let r = &mut state.receipts[index];
                        r.status = if r.attempts < 3 { "queued" } else { "failed" }.into();
                        r.issues = vec![error.to_string()];
                        r.lease = None;
                        r.lease_until = None;
                        r.next_attempt_at = Some(
                            Utc::now() + chrono::Duration::seconds(60 * i64::from(r.attempts)),
                        );
                        r.revision += 1;
                    }
                }
                Ok(())
            })
            .await?;
            if result.is_ok() {
                self.announce(&ctx, &receipt.id).await;
            }
            return Ok(());
        }
        let import = self
            .mutate(&ctx, &[], |state| {
                let catalog = state.catalog.clone();
                if let Some(job) = state.imports.iter_mut().find(|i| {
                    i.status == "queued"
                        || (i.status == "processing" && i.lease_until.is_none_or(|at| at < now))
                }) {
                    job.status = "processing".into();
                    job.lease = Some(lease.clone());
                    job.lease_until = Some(now + chrono::Duration::minutes(3));
                    return Ok(Some((job.clone(), catalog)));
                }
                Ok(None)
            })
            .await?;
        if let Some((job, catalog)) = import {
            let result = self.model.list(&job.text, &catalog).await;
            self.mutate(&ctx, &[], |state| {
                let Some(index) = state
                    .imports
                    .iter()
                    .position(|i| i.id == job.id && i.lease.as_deref() == Some(&lease))
                else {
                    return Ok(());
                };
                match &result {
                    Ok(products) => {
                        let mut candidate = state.clone();
                        let mut failure = None;
                        for (i, p) in products.iter().enumerate() {
                            let id = Uuid::new_v5(
                                &Uuid::NAMESPACE_URL,
                                format!("shopping/import/{}/{i}", job.id).as_bytes(),
                            );
                            if let Err(error) =
                                candidate.add_classified(id.to_string(), p, &job.author)
                            {
                                failure = Some(error);
                                break;
                            }
                        }
                        if let Some(error) = failure {
                            state.imports[index].status = "failed".into();
                            state.imports[index].error = Some(error.to_string());
                        } else {
                            candidate.imports[index].status = "done".into();
                            *state = candidate;
                        }
                    }
                    Err(e) => {
                        state.imports[index].status = "failed".into();
                        state.imports[index].error = Some(e.to_string());
                    }
                }
                Ok(())
            })
            .await?;
        } else {
            // Classify new manually entered products in a bounded batch, without delaying typing.
            let (_, state) = self.store.read(&scope).await?;
            if state.classification_next_at.is_some_and(|at| at > now) {
                return Ok(());
            }
            let batch: Vec<_> = state
                .catalog
                .iter()
                .filter(|p| !p.classified)
                .take(12)
                .cloned()
                .collect();
            if !batch.is_empty() {
                let text = batch
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                let claimed = self
                    .mutate(&ctx, &[], |s| {
                        if s.classification_next_at.is_some_and(|at| at > now) {
                            return Ok(false);
                        }
                        s.classification_next_at = Some(now + chrono::Duration::minutes(5));
                        Ok(true)
                    })
                    .await?;
                if !claimed {
                    return Ok(());
                }
                if let Ok(classified) = self.model.list(&text, &state.catalog).await {
                    self.mutate(&ctx, &[], |s| {
                        s.reconcile_classification(&batch, &classified);
                        s.classification_next_at = if s
                            .catalog
                            .iter()
                            .any(|p| batch.iter().any(|b| b.id == p.id) && !p.classified)
                        {
                            Some(now + chrono::Duration::minutes(5))
                        } else {
                            None
                        };
                        Ok(())
                    })
                    .await?;
                }
            }
        }
        Ok(())
    }
}
