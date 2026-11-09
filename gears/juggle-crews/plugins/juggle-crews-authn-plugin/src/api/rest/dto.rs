//! Wire shapes of the sign-in routes.

/// How this server signs people in.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct SessionConfigDto {
    /// `google` or `demo`.
    pub sign_in: String,
    /// The demo characters; empty outside demo mode.
    pub demo_users: Vec<DemoUserDto>,
}

/// A demo character on the sign-in screen.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct DemoUserDto {
    /// Character key to sign in with.
    pub key: String,
    /// Display name.
    pub name: String,
    /// Name of the character's crew.
    pub crew: Option<String>,
    /// One line about the character.
    pub tagline: String,
}

/// Sign in as a demo character.
#[derive(Debug)]
#[toolkit_macros::api_dto(request)]
pub struct DemoSessionRequest {
    /// Character key, e.g. `pip`.
    pub user: String,
}

/// A demo bearer token, valid until the server restarts.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct DemoTokenDto {
    /// `demo.<character>.<signature>`.
    pub token: String,
}
