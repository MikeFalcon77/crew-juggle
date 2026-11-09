use super::{Error, Result};
/// Rejects unsupported delivery hosts, malformed keys and unsafe endpoints.
pub fn validate_subscription(endpoint: &str, p256dh: &str, auth: &str) -> Result<()> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let url =
        url::Url::parse(endpoint).map_err(|_| Error::Invalid("Invalid push endpoint".into()))?;
    let host = url.host_str().unwrap_or_default();
    let permitted = [
        "fcm.googleapis.com",
        "updates.push.services.mozilla.com",
        "web.push.apple.com",
    ]
    .contains(&host)
        || host.ends_with(".notify.windows.com");
    if !permitted
        || url.scheme() != "https"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || endpoint.len() > 2000
        || url.fragment().is_some()
    {
        return Err(Error::Invalid("This push service is not supported".into()));
    }
    if URL_SAFE_NO_PAD
        .decode(p256dh)
        .map_or(true, |key| key.len() != 65 || key[0] != 4)
        || URL_SAFE_NO_PAD
            .decode(auth)
            .map_or(true, |key| key.len() != 16)
    {
        return Err(Error::Invalid("Invalid push subscription keys".into()));
    }
    Ok(())
}
