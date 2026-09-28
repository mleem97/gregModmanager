//! Authentication models (legacy BetterAuth + browser session flow).

use serde::{Deserialize, Serialize};

/// Legacy email/password login request.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
}

/// Legacy auth response carrying a session token.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    #[serde(default)]
    pub token: String,
}

/// Legacy user info.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub name: String,
}

/// Browser-flow session lifecycle.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionState {
    /// No session.
    #[default]
    SignedOut,
    /// Waiting for the browser callback.
    AwaitingCallback,
    /// Active session with a token.
    SignedIn,
    /// Session expired or revoked.
    Expired,
}

/// Identity attached to an active session.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountIdentity {
    #[serde(default)]
    pub subject_id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub tenant: String,
}

/// Active browser-flow session.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSession {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub identity: AccountIdentity,
}

/// Token exchange request after the browser callback.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenExchangeRequest {
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub nonce: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

/// Token exchange response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenExchangeResponse {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub user: ExchangeUser,
}

/// User embedded in the exchange response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeUser {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub subject_id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub tenant: String,
}

/// Session manager contract (UI-agnostic state machine).
pub trait SessionManager {
    /// Current lifecycle state.
    fn state(&self) -> SessionState;
    /// Current session, if signed in.
    fn current_session(&self) -> Option<ActiveSession>;
    /// Forget the session.
    fn logout(&mut self);
}
