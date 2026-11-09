use super::{AvatarNormalizer, AvatarStore, Error, PushTransport, Result, Service, Subscription};
use juggle_crews_sdk::is_worker;
use juggle_routines_sdk::{
    CrewPerson, DeliveryOutcome, Notification, NotificationKind, RoutinesClientV1, RoutinesError,
};
use std::sync::Arc;
use toolkit_security::SecurityContext;

#[toolkit_macros::domain_model]
/// Crew use cases composed from domain ports.
pub struct Application {
    /// Crew-scoped routines use cases.
    pub service: Service,
    /// Where photos are stored.
    pub avatars: Arc<dyn AvatarStore>,
    /// Validates image bytes before a photo replacement is committed.
    pub normalizer: Arc<dyn AvatarNormalizer>,
    /// Web Push encryption and delivery.
    pub push: Arc<dyn PushTransport>,
    /// Public VAPID key browsers use to subscribe.
    pub vapid_public_key: String,
}
impl Application {
    /// The caller's account id, if they belong to the crew.
    pub async fn actor(&self, ctx: &SecurityContext) -> Result<String> {
        Ok(self.service.member(ctx).await?.uid)
    }

    /// Registers this account's device without duplicating endpoints.
    pub async fn subscribe(
        &self,
        ctx: &SecurityContext,
        endpoint: String,
        p256dh: String,
        auth: String,
    ) -> Result<String> {
        let uid = self.actor(ctx).await?;
        super::subscriptions::validate_subscription(&endpoint, &p256dh, &auth)?;
        use aws_lc_rs::digest::{SHA256, digest};
        use base64::Engine;
        let id = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(digest(&SHA256, endpoint.as_bytes()).as_ref());
        let subscription = Subscription {
            id: id.clone(),
            uid,
            endpoint,
            p256dh,
            auth,
        };
        self.service
            .mutate(ctx, |state| {
                state.subscriptions.retain(|s| s.id != id);
                state.subscriptions.push(subscription.clone());
                Ok(())
            })
            .await?;
        Ok(id)
    }

    /// Removes a device only when it belongs to the caller.
    pub async fn unsubscribe(&self, ctx: &SecurityContext, id: &str) -> Result<()> {
        let uid = self.actor(ctx).await?;
        self.service
            .mutate(ctx, |state| {
                state.subscriptions.retain(|s| s.id != id || s.uid != uid);
                Ok(())
            })
            .await
    }

    /// Sends a test notification to one of the caller's own devices.
    pub async fn test_push(&self, ctx: &SecurityContext, id: &str) -> Result<()> {
        let uid = self.actor(ctx).await?;
        let (_, state) = self.service.read(ctx).await?;
        let sub = state
            .subscriptions
            .iter()
            .find(|s| s.id == id && s.uid == uid)
            .ok_or(Error::NotFound)?;
        let message = Notification {
            title: "crew-juggle · test".into(),
            body: "Notifications work on this device 🎉".into(),
            tag: "routines-test".into(),
            url: "/today".into(),
            kind: NotificationKind::Routines,
            ttl_seconds: 300,
        };
        if self.push.send(sub, &message).await? != DeliveryOutcome::Sent {
            return Err(Error::Invalid(
                "The push service refused this device. Turn notifications on again.".into(),
            ));
        }
        Ok(())
    }

    /// Photo metadata under the caller's scope.
    pub async fn avatar_metadata(
        &self,
        ctx: &SecurityContext,
    ) -> Result<std::collections::HashMap<String, super::AvatarMeta>> {
        let scope = self.service.scope(ctx).await?;
        self.avatars.metadata(&scope).await
    }

    /// One person's photo under the caller's scope.
    pub async fn avatar_image(&self, ctx: &SecurityContext, id: &str) -> Result<Vec<u8>> {
        let scope = self.service.scope(ctx).await?;
        self.avatars.image(&scope, id).await
    }

    /// Atomically changes a photo using the caller's identity and scope.
    pub async fn change_avatar(
        &self,
        ctx: &SecurityContext,
        id: &str,
        request: &super::AvatarEdit,
        image: Option<Vec<u8>>,
    ) -> Result<super::AvatarMeta> {
        let actor = self.actor(ctx).await?;
        // Make sure the crew state exists before the avatar transaction locks it.
        self.service.read(ctx).await?;
        let scope = self.service.scope(ctx).await?;
        self.avatars
            .change(&scope, id, &actor, request, image)
            .await
    }
}

fn valid_for_devices(message: &Notification) -> bool {
    let path = match message.kind {
        NotificationKind::Routines => "/today",
        NotificationKind::Shopping => "/shopping",
    };
    // Only in-app navigation reaches a device; no arbitrary URLs.
    (message.url == path || message.url.starts_with(&format!("{path}?")))
        && !message.url.contains(['\\', '\r', '\n'])
        && message.body.chars().count() <= 2000
        && message.title.chars().count() <= 200
        && (1..=86400).contains(&message.ttl_seconds)
}

#[async_trait::async_trait]
impl RoutinesClientV1 for Application {
    async fn people(
        &self,
        ctx: &SecurityContext,
    ) -> std::result::Result<Vec<CrewPerson>, RoutinesError> {
        if !is_worker(ctx) && self.actor(ctx).await.is_err() {
            return Err(RoutinesError::Forbidden);
        }
        let (_, state) = self
            .service
            .read(ctx)
            .await
            .map_err(|_| RoutinesError::Unavailable)?;
        Ok(state
            .people
            .into_iter()
            .map(|p| CrewPerson {
                id: p.id,
                name: p.name,
                account: p.account,
            })
            .collect())
    }

    async fn notify_crew(
        &self,
        ctx: &SecurityContext,
        message: Notification,
        skip_account: Option<String>,
    ) -> std::result::Result<usize, RoutinesError> {
        if !is_worker(ctx) {
            return Err(RoutinesError::Forbidden);
        }
        if !valid_for_devices(&message) {
            return Err(RoutinesError::Invalid);
        }
        let (_, state) = self
            .service
            .read(ctx)
            .await
            .map_err(|_| RoutinesError::Unavailable)?;
        let mut sent = 0;
        let mut gone = vec![];
        for sub in state
            .subscriptions
            .iter()
            .filter(|s| Some(&s.uid) != skip_account.as_ref())
        {
            match self.push.send(sub, &message).await {
                Ok(DeliveryOutcome::Sent) => sent += 1,
                Ok(DeliveryOutcome::SubscriptionGone) => gone.push(sub.id.clone()),
                _ => {}
            }
        }
        if !gone.is_empty() {
            self.service
                .mutate(ctx, |state| {
                    state.subscriptions.retain(|s| !gone.contains(&s.id));
                    Ok(())
                })
                .await
                .map_err(|_| RoutinesError::Unavailable)?;
        }
        Ok(sent)
    }
}
