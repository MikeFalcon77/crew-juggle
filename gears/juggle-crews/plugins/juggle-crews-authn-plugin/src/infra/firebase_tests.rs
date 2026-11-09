use super::*;
use jsonwebtoken::{EncodingKey, Header, encode};
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};
use serde::Serialize;
use std::sync::OnceLock;

const PROJECT: &str = "demo-project";
const ACCOUNT_UID: &str = "firebase-uid-of-pip";
const ACCOUNT_EMAIL: &str = "pip@example.com";

/// A keypair of our own, so these tests exercise the rules rather than the
/// network. Google's key is not what any of them is about.
fn generate_keys() -> (String, String) {
    let private = RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048).expect("test RSA key");
    let public = private
        .to_public_key()
        .to_public_key_pem(LineEnding::LF)
        .expect("test public PEM");
    let private = private
        .to_pkcs8_pem(LineEnding::LF)
        .expect("test private PEM");
    (private.to_string(), public)
}
fn test_keys() -> &'static (String, String) {
    static KEYS: OnceLock<(String, String)> = OnceLock::new();
    KEYS.get_or_init(generate_keys)
}

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    aud: String,
    iss: String,
    exp: i64,
    iat: i64,
    email: String,
    email_verified: bool,
}

fn verifier() -> FirebaseVerifier {
    // Not `new`: that builds an HTTP client, which needs a runtime and is
    // no part of what is under test here.
    FirebaseVerifier {
        project_id: PROJECT.to_owned(),
        allowlist: None,
        http: None,
        keys: RwLock::new(None),
    }
}

fn key() -> DecodingKey {
    DecodingKey::from_rsa_pem(test_keys().1.as_bytes()).expect("test public key")
}

fn token_for(uid: &str) -> String {
    token_with(uid, ACCOUNT_EMAIL, true)
}

fn token_with(uid: &str, email: &str, email_verified: bool) -> String {
    signed(
        uid,
        email,
        email_verified,
        PROJECT,
        &format!("https://securetoken.google.com/{PROJECT}"),
        3600,
    )
}

fn signed(
    uid: &str,
    email: &str,
    email_verified: bool,
    aud: &str,
    iss: &str,
    expires_in: i64,
) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = TestClaims {
        sub: uid.to_owned(),
        aud: aud.to_owned(),
        iss: iss.to_owned(),
        exp: now + expires_in,
        iat: now - 10,
        email: email.to_owned(),
        email_verified,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key".to_owned());
    encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(test_keys().0.as_bytes()).expect("test private key"),
    )
    .expect("signed")
}

fn private(verifier: FirebaseVerifier, emails: &[&str]) -> FirebaseVerifier {
    FirebaseVerifier {
        allowlist: Some(normalize(emails.iter().map(|e| (*e).to_owned()).collect())),
        ..verifier
    }
}

#[test]
fn any_verified_google_account_gets_an_identity_with_open_sign_up() {
    let identity = verifier()
        .check(&token_for("some-new-google-user"), &key())
        .expect("open sign-up identifies everyone; crews decide access");
    assert_eq!(identity.uid, "some-new-google-user");
    assert_eq!(identity.email, ACCOUNT_EMAIL);
}

/// An identity is not access: a verified stranger reaches only the crews API,
/// because the PDP grants crew data to members alone. That rule is tested in
/// `juggle-crews-sdk::policy`; this module must only never invent an identity.
#[test]
fn an_unverified_email_is_refused_even_with_open_sign_up() {
    let refusal = verifier()
        .check(&token_with(ACCOUNT_UID, ACCOUNT_EMAIL, false), &key())
        .expect_err("an unverified address must not be shown to crewmates");
    assert!(
        matches!(&refusal, Refusal::NotAllowed { uid, email: None } if uid == ACCOUNT_UID),
        "{refusal:?}"
    );
}

#[test]
fn a_private_deployment_refuses_addresses_outside_its_allowlist() {
    let refusal = private(verifier(), &["Someone.Else@Example.com"])
        .check(&token_for(ACCOUNT_UID), &key())
        .expect_err("a stranger with a valid token must be refused");
    assert!(
        matches!(&refusal, Refusal::NotAllowed { uid, .. } if uid == ACCOUNT_UID),
        "{refusal:?}"
    );
}

#[test]
fn a_private_deployment_admits_its_allowlist_case_insensitively() {
    let identity = private(verifier(), &["PIP@example.com"])
        .check(&token_for(ACCOUNT_UID), &key())
        .expect("a listed address gets in");
    assert_eq!(identity.uid, ACCOUNT_UID);
}

/// Signed by the same Google key, minted for a different project.
#[test]
fn a_token_for_another_firebase_project_is_refused() {
    let other = signed(
        ACCOUNT_UID,
        ACCOUNT_EMAIL,
        true,
        "somebody-elses-project",
        "https://securetoken.google.com/somebody-elses-project",
        3600,
    );
    assert!(matches!(
        verifier().check(&other, &key()),
        Err(Refusal::NotAValidToken(_))
    ));
}

/// The right project, the right admin, and the wrong issuer: a token minted
/// by something that is not Firebase.
#[test]
fn a_token_from_the_wrong_issuer_is_refused() {
    let forged = signed(
        ACCOUNT_UID,
        ACCOUNT_EMAIL,
        true,
        PROJECT,
        "https://example.com/",
        3600,
    );
    assert!(matches!(
        verifier().check(&forged, &key()),
        Err(Refusal::NotAValidToken(_))
    ));
}

#[test]
fn an_expired_id_token_is_refused() {
    let stale = signed(
        ACCOUNT_UID,
        ACCOUNT_EMAIL,
        true,
        PROJECT,
        &format!("https://securetoken.google.com/{PROJECT}"),
        // Past the default 60s leeway, so this is expiry rather than skew.
        -600,
    );
    assert!(matches!(
        verifier().check(&stale, &key()),
        Err(Refusal::NotAValidToken(_))
    ));
}

/// A token signed by a key that is not the one we verify with.
#[test]
fn a_token_signed_by_an_unknown_key_is_refused() {
    let wrong_key = DecodingKey::from_rsa_pem(generate_keys().1.as_bytes()).expect("other key");
    assert!(matches!(
        verifier().check(&token_for(ACCOUNT_UID), &wrong_key),
        Err(Refusal::NotAValidToken(_))
    ));
}

#[test]
fn the_key_url_is_the_firebase_one_not_the_oauth_one() {
    // These differ by one path and fail in opposite directions: the OAuth
    // set verifies nothing Firebase signs, so every genuine admin token
    // would be refused with a signature error.
    assert!(JWK_URL.contains("securetoken@system.gserviceaccount.com"));
    assert!(!JWK_URL.contains("oauth2"));
}

#[test]
fn a_cache_lifetime_is_read_from_the_response() {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::CACHE_CONTROL,
        "public, max-age=19008, must-revalidate, no-transform"
            .parse()
            .unwrap(),
    );
    assert_eq!(max_age(&headers), Some(Duration::from_secs(19_008)));
}

#[test]
fn a_missing_or_useless_cache_header_falls_back_rather_than_caching_forever() {
    assert_eq!(max_age(&http::HeaderMap::new()), None);

    let mut zero = http::HeaderMap::new();
    zero.insert(http::header::CACHE_CONTROL, "max-age=0".parse().unwrap());
    assert_eq!(max_age(&zero), None);
}
