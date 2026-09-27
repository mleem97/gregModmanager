//! Auth HTTP clients (`AuthApiClient`, `BetterAuthService`,
//! `GitVerificationService` port).
//!
//! All calls try each configured base URL in order (failover).

use greg_core::models::{
    AccountIdentity, ActiveSession, AuthResponse, LoginRequest, TokenExchangeRequest,
    TokenExchangeResponse, UserInfo,
};

use crate::error::{ModStoreError, Result};

/// Callback redirect URI registered for the desktop flow.
pub const AUTH_CALLBACK_REDIRECT_URI: &str = "greg://auth/callback";

/// Browser-flow client: login URL, token exchange, logout.
#[derive(Debug, Clone)]
pub struct AuthApiClient {
    http: reqwest::Client,
    login_url_formats: Vec<String>,
    api_base_urls: Vec<String>,
}

impl AuthApiClient {
    /// Creates the client with login-URL formats
    /// (`{redirect}`, `{requestId}` placeholders) and API base URLs.
    pub fn new(login_url_formats: Vec<String>, api_base_urls: Vec<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            login_url_formats,
            api_base_urls,
        }
    }

    /// Builds the browser login URL from the first reachable candidate.
    pub async fn login_url(&self, request_id: &str) -> Result<String> {
        let redirect = percent_encode(AUTH_CALLBACK_REDIRECT_URI);
        let mut last: Option<ModStoreError> = None;
        for format in &self.login_url_formats {
            let candidate = format.replacen("{0}", &redirect, 1).replacen(
                "{1}",
                &percent_encode(request_id),
                1,
            );
            // Positional fallback for `{redirect}` / `{requestId}` styles.
            let candidate = candidate
                .replace("{redirect}", &redirect)
                .replace("{requestId}", &percent_encode(request_id));
            match self.http.get(&candidate).send().await {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                    return Ok(candidate)
                }
                Ok(resp) => {
                    last = Some(ModStoreError::Request(format!("status {}", resp.status())))
                }
                Err(e) => last = Some(ModStoreError::Request(e.to_string())),
            }
        }
        Err(last.unwrap_or_else(|| ModStoreError::NotAvailable("no login URL reachable".into())))
    }

    /// Exchanges a browser-callback code for a session.
    pub async fn exchange_callback_code(
        &self,
        request: TokenExchangeRequest,
    ) -> Result<Option<ActiveSession>> {
        let mut request = request;
        if request.redirect_uri.is_none() {
            request.redirect_uri = Some(AUTH_CALLBACK_REDIRECT_URI.to_string());
        }
        for base in &self.api_base_urls {
            let url = format!("{}/token", base.trim_end_matches('/'));
            match self.http.post(&url).json(&request).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<TokenExchangeResponse>().await {
                        Ok(token) if !token.access_token.is_empty() => {
                            let user = token.user;
                            let display = if user.display_name.is_empty() {
                                if user.name.is_empty() {
                                    "User".to_string()
                                } else {
                                    user.name.clone()
                                }
                            } else {
                                user.display_name.clone()
                            };
                            return Ok(Some(ActiveSession {
                                access_token: token.access_token,
                                session_id: token.session_id,
                                identity: AccountIdentity {
                                    subject_id: if user.subject_id.is_empty() {
                                        user.id.clone()
                                    } else {
                                        user.subject_id.clone()
                                    },
                                    email: user.email.clone(),
                                    display_name: display,
                                    avatar_url: if user.avatar_url.is_empty() {
                                        None
                                    } else {
                                        Some(user.avatar_url.clone())
                                    },
                                    roles: if user.roles.is_empty() {
                                        vec!["user".to_string()]
                                    } else {
                                        user.roles.clone()
                                    },
                                    tenant: user.tenant.clone(),
                                },
                            }));
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
        Ok(None)
    }

    /// Ends a session (best-effort across base URLs).
    pub async fn end_session(&self, access_token: &str) -> bool {
        for base in &self.api_base_urls {
            let url = format!("{}/logout", base.trim_end_matches('/'));
            if let Ok(resp) = self.http.post(&url).bearer_auth(access_token).send().await {
                if resp.status().is_success() {
                    return true;
                }
            }
        }
        false
    }
}

/// Legacy email/password client (BetterAuth: `/sign-in/email`, `/get-session`).
#[derive(Debug, Clone)]
pub struct BetterAuthClient {
    http: reqwest::Client,
    base_urls: Vec<String>,
}

impl BetterAuthClient {
    /// Creates the client.
    pub fn new(base_urls: Vec<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_urls,
        }
    }

    /// Signs in with email + password.
    pub async fn login(&self, email: &str, password: &str) -> Result<Option<AuthResponse>> {
        let payload = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        for base in &self.base_urls {
            let url = format!("{}/sign-in/email", base.trim_end_matches('/'));
            match self.http.post(&url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => match resp.json::<AuthResponse>().await {
                    Ok(auth) => return Ok(Some(auth)),
                    Err(_) => continue,
                },
                _ => continue,
            }
        }
        Ok(None)
    }

    /// Verifies a session token.
    pub async fn verify_session(&self, token: &str) -> bool {
        for base in &self.base_urls {
            let url = format!("{}/get-session", base.trim_end_matches('/'));
            if let Ok(resp) = self.http.get(&url).bearer_auth(token).send().await {
                if resp.status().is_success() {
                    if let Ok(user) = resp.json::<UserInfo>().await {
                        return !user.id.is_empty() || !user.email.is_empty();
                    }
                    return true;
                }
            }
        }
        false
    }
}

/// Verifies a user token against the git server (`GitVerificationService` port).
pub async fn verify_git_user(git_server_url: &str, api_token: &str) -> bool {
    let client = reqwest::Client::new();
    // Bearer header first, access_token query as fallback.
    if let Ok(resp) = client
        .get(git_server_url)
        .bearer_auth(api_token)
        .send()
        .await
    {
        if resp.status().is_success() {
            return true;
        }
    }
    let url = format!(
        "{}{}access_token={}",
        git_server_url,
        if git_server_url.contains('?') {
            "&"
        } else {
            "?"
        },
        percent_encode(api_token)
    );
    matches!(
        client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success()),
        Ok(true)
    )
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes() {
        assert_eq!(percent_encode("a b/c"), "a%20b%2Fc");
        assert_eq!(
            percent_encode("greg://auth/callback"),
            "greg%3A%2F%2Fauth%2Fcallback"
        );
    }
}
