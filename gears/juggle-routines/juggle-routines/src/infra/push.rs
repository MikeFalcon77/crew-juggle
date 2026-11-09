use crate::domain::{Error, PushTransport, Result, Subscription};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use juggle_routines_sdk::{DeliveryOutcome, Notification, NotificationKind};
use std::time::Duration;
use web_push_native::{
    Auth, WebPushBuilder,
    jwt_simple::algorithms::ES256KeyPair,
    p256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint},
};

/// Encrypted Web Push adapter using the configured VAPID identity.
pub struct Push {
    key_pair: ES256KeyPair,
    /// Public VAPID key for browser subscription creation.
    pub public_key: String,
    /// `mailto:` or `https:` contact push services may use about this sender.
    subject: String,
    client: reqwest::Client,
}
impl Push {
    /// Loads the VAPID identity from the gear config. Outside production, a
    /// missing identity is replaced by a throwaway one, so local runs and the
    /// demo can still deliver notifications (until the next restart).
    pub fn new(config: &crate::config::RoutinesConfig) -> anyhow::Result<Self> {
        let private_key = config.vapid_private_key.trim().to_owned();
        let public_key = config.vapid_public_key.trim().to_owned();
        let (key_pair, public_key) = if private_key.is_empty() || public_key.is_empty() {
            anyhow::ensure!(
                !config.production,
                "VAPID_PRIVATE_KEY and VAPID_PUBLIC_KEY are required in production"
            );
            tracing::warn!("no VAPID keys configured; using throwaway keys for this run");
            let key_pair = ES256KeyPair::generate();
            // Browsers want the uncompressed point; jwt-simple hands out the compressed one.
            let point = PublicKey::from_sec1_bytes(&key_pair.public_key().to_bytes())?
                .to_encoded_point(false);
            let public = URL_SAFE_NO_PAD.encode(point.as_bytes());
            (key_pair, public)
        } else {
            (
                ES256KeyPair::from_bytes(&URL_SAFE_NO_PAD.decode(private_key)?)?,
                public_key,
            )
        };
        let subject = Some(config.vapid_subject.trim().to_owned())
            .filter(|s| s.starts_with("mailto:") || s.starts_with("https://"))
            .unwrap_or_else(|| "https://github.com/MikeFalcon77/crew-juggle".into());
        let client = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(25))
            .build()?;
        Ok(Self {
            key_pair,
            public_key,
            subject,
            client,
        })
    }
    async fn send_message(
        &self,
        subscription: &Subscription,
        message: &Notification,
    ) -> anyhow::Result<u16> {
        let builder = WebPushBuilder::new(
            subscription.endpoint.parse()?,
            PublicKey::from_sec1_bytes(&URL_SAFE_NO_PAD.decode(&subscription.p256dh)?)?,
            Auth::clone_from_slice(&URL_SAFE_NO_PAD.decode(&subscription.auth)?),
        )
        .with_vapid(&self.key_pair, &self.subject);
        let payload =
            serde_json::json!({"title":message.title,"body":message.body,"tag":message.tag,"url":message.url,"kind":match message.kind { NotificationKind::Routines => "routines", NotificationKind::Shopping => "shopping" }}).to_string();
        let request = builder.build(payload)?;
        let (parts, body) = request.into_parts();
        let response = self
            .client
            .request(parts.method, parts.uri.to_string())
            .headers(parts.headers)
            .header("TTL", message.ttl_seconds)
            .body(body)
            .send()
            .await?;
        Ok(response.status().as_u16())
    }
}

#[async_trait]
impl PushTransport for Push {
    async fn send(
        &self,
        subscription: &Subscription,
        message: &Notification,
    ) -> Result<DeliveryOutcome> {
        let status = self
            .send_message(subscription, message)
            .await
            .map_err(Error::Storage)?;
        Ok(match status {
            200..=299 => DeliveryOutcome::Sent,
            404 | 410 => DeliveryOutcome::SubscriptionGone,
            _ => DeliveryOutcome::Retry,
        })
    }
}
