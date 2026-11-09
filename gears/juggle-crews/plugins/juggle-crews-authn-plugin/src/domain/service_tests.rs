use super::*;
use async_trait::async_trait;
use juggle_crews_sdk::{AccountIdentity, CrewsError, MembershipKind, lobby_tenant, subject_for};
use uuid::Uuid;

struct OneToken;

#[async_trait]
impl IdentityVerifier for OneToken {
    async fn verify(&self, token: &str) -> Result<AccountIdentity, Refusal> {
        match token {
            "good" => Ok(AccountIdentity {
                uid: "uid-pip".into(),
                email: "pip@example.test".into(),
                name: None,
            }),
            "keys-down" => Err(Refusal::KeysUnavailable("offline".into())),
            "outsider" => Err(Refusal::NotAllowed {
                uid: "uid-x".into(),
                email: None,
            }),
            _ => Err(Refusal::NotAValidToken("bad".into())),
        }
    }
}

/// A directory where nobody has a crew yet, or which is down.
struct Directory {
    down: bool,
}

#[async_trait]
impl CrewDirectoryV1 for Directory {
    async fn is_member(&self, _: Uuid, _: Uuid) -> Result<bool, CrewsError> {
        Ok(false)
    }
    async fn sign_in(&self, identity: AccountIdentity) -> Result<AccountMembership, CrewsError> {
        if self.down {
            return Err(CrewsError::Unavailable);
        }
        Ok(AccountMembership {
            subject_id: subject_for(&identity.uid),
            tenant_id: lobby_tenant(&identity.uid),
            kind: MembershipKind::Guest,
        })
    }
}

fn sign_in(down: bool) -> SignIn {
    SignIn::new(Arc::new(OneToken), Arc::new(Directory { down }))
}

#[tokio::test]
async fn a_verified_token_lands_where_the_directory_says() {
    let membership = sign_in(false).sign_in("good").await.unwrap();
    assert_eq!(membership.subject_id, subject_for("uid-pip"));
    assert_eq!(membership.kind, MembershipKind::Guest);
}

#[tokio::test]
async fn refusals_are_one_answer_and_outages_are_another() {
    let service = sign_in(false);
    for token in ["bad", "outsider"] {
        assert!(matches!(
            service.sign_in(token).await,
            Err(SignInError::Refused)
        ));
    }
    assert!(matches!(
        service.sign_in(&"x".repeat(9000)).await,
        Err(SignInError::Refused)
    ));
    assert!(matches!(
        service.sign_in("keys-down").await,
        Err(SignInError::Unavailable)
    ));
    assert!(matches!(
        sign_in(true).sign_in("good").await,
        Err(SignInError::Unavailable)
    ));
}
