//! The OpenAI upstream in oagw: created at start-up, reused after restarts.
use juggle_crews_sdk::integration_context;
use oagw_sdk::{
    APIKEY_AUTH_PLUGIN_ID, AuthConfig, CreateRouteRequest, CreateUpstreamRequest, Endpoint,
    HTTP_PROTOCOL_ID, HttpMatch, HttpMethod, ListQuery, MatchRules, PathSuffixMode, Scheme, Server,
    ServiceGatewayClientV1, SharingMode, Upstream,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use toolkit_canonical_errors::CanonicalError;

/// Where OpenAI lives and which credstore secret carries its key.
pub struct OpenAiUpstream {
    scheme: Scheme,
    host: String,
    port: u16,
    secret_ref: String,
    ready: AtomicBool,
}

impl OpenAiUpstream {
    /// The upstream at `base_url` (`https://host[:port]`), authenticated with
    /// the credstore secret `secret_ref`.
    pub fn new(base_url: &str, secret_ref: &str) -> anyhow::Result<Self> {
        let uri: http::Uri = base_url.trim().parse()?;
        let (scheme, default_port) = match uri.scheme_str() {
            Some("https") => (Scheme::Https, 443),
            Some("http") => (Scheme::Http, 80),
            _ => anyhow::bail!("openai_base_url must start with https:// or http://"),
        };
        let host = uri
            .host()
            .ok_or_else(|| anyhow::anyhow!("openai_base_url needs a host"))?
            .to_owned();
        Ok(Self {
            scheme,
            host,
            port: uri.port_u16().unwrap_or(default_port),
            secret_ref: secret_ref.trim().to_owned(),
            ready: AtomicBool::new(false),
        })
    }

    /// True once oagw has the upstream and its route.
    pub fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    /// The proxy path of the Responses API: `/{alias}/v1/responses`.
    pub fn responses_uri(&self) -> String {
        format!("/{}/v1/responses", self.host)
    }

    /// Creates the upstream and its route, or reuses them after a restart.
    /// Fails while the key is not in credstore yet; the caller retries.
    pub async fn provision(
        &self,
        gateway: &Arc<dyn ServiceGatewayClientV1>,
    ) -> Result<(), CanonicalError> {
        let ctx = integration_context();
        let upstream = match gateway.create_upstream(ctx.clone(), self.request()).await {
            Ok(upstream) => upstream,
            Err(CanonicalError::AlreadyExists { .. }) => self.existing(gateway).await?,
            Err(error) => return Err(error),
        };
        let route = MatchRules {
            http: Some(HttpMatch {
                methods: vec![HttpMethod::Post],
                path: "/v1/responses".into(),
                query_allowlist: vec![],
                path_suffix_mode: PathSuffixMode::Disabled,
            }),
            grpc: None,
        };
        match gateway
            .create_route(
                ctx,
                CreateRouteRequest::builder(upstream.id, route)
                    .enabled(true)
                    .build(),
            )
            .await
        {
            Ok(_) | Err(CanonicalError::AlreadyExists { .. }) => {}
            Err(error) => return Err(error),
        }
        self.ready.store(true, Ordering::Release);
        Ok(())
    }

    fn request(&self) -> CreateUpstreamRequest {
        let server = Server {
            endpoints: vec![Endpoint {
                scheme: self.scheme,
                host: self.host.clone(),
                port: self.port,
            }],
        };
        let auth = HashMap::from([
            ("header".to_owned(), "authorization".to_owned()),
            ("prefix".to_owned(), "Bearer ".to_owned()),
            (
                "secret_ref".to_owned(),
                format!("cred://{}", self.secret_ref),
            ),
        ]);
        CreateUpstreamRequest::builder(server, HTTP_PROTOCOL_ID)
            .alias(&self.host)
            .auth(AuthConfig {
                plugin_type: APIKEY_AUTH_PLUGIN_ID.to_owned(),
                sharing: SharingMode::Private,
                config: Some(auth),
            })
            .enabled(true)
            .build()
    }

    async fn existing(
        &self,
        gateway: &Arc<dyn ServiceGatewayClientV1>,
    ) -> Result<Upstream, CanonicalError> {
        let found = gateway
            .list_upstreams(integration_context(), &ListQuery::default())
            .await?
            .into_iter()
            .find(|u| u.alias == self.host);
        found.ok_or_else(|| {
            CanonicalError::internal("the OpenAI upstream exists but cannot be listed").create()
        })
    }
}
