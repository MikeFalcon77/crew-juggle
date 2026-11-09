use super::*;

#[test]
fn a_minted_token_verifies_only_on_the_server_that_minted_it() {
    let (here, elsewhere) = (DemoTokens::new().unwrap(), DemoTokens::new().unwrap());
    let token = here.mint("pip").unwrap();
    assert_eq!(here.identity_of(&token).unwrap().uid, "demo-pip");
    assert!(elsewhere.identity_of(&token).is_none());
}

#[test]
fn unknown_characters_and_edited_tokens_are_refused() {
    let tokens = DemoTokens::new().unwrap();
    assert!(tokens.mint("mallory").is_none());
    let token = tokens.mint("zed").unwrap();
    assert!(tokens.identity_of(&token.replace("zed", "pip")).is_none());
    assert!(tokens.identity_of("demo.zed.00").is_none());
    assert!(tokens.identity_of("eyJhbGciOi.not.a.demo").is_none());
}
