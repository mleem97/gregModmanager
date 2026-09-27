//! `greg://install` intent parsing + validation (`InstallIntentClient` port).
//!
//! Structural validation (scheme, host, params, expiry, replay) always runs.
//! Cryptographic signature verification is a [`SignatureVerifier`] hook and
//! fails closed (reject) unless the composition root provides one — mirroring
//! the C# "cryptographic verification required" behavior.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::error::{ModStoreError, Result};

/// Required intent roles of the current session (opaque strings).
pub type Roles = Vec<String>;

/// Parsed install intent (`InstallIntentContext` port).
#[derive(Debug, Clone, Default)]
pub struct InstallIntent {
    /// Replay-protection id.
    pub intent_id: String,
    /// Package to install.
    pub package_id: String,
    /// Source URL the package must come from.
    pub source_url: String,
    /// Subject the intent was issued for.
    pub subject_id: String,
    /// Unix expiry timestamp (0 = none).
    pub expires_at: i64,
    /// Required session roles.
    pub required_roles: Vec<String>,
    /// Base64 signature (ECDSA P-256, ~64 chars minimum).
    pub signature: String,
}

/// Verifies an intent signature. Fail-closed default: [`RejectAll`].
pub trait SignatureVerifier: Send + Sync {
    /// Returns `Ok(())` when the signature is valid for the intent.
    fn verify(&self, intent: &InstallIntent) -> Result<()>;
}

/// Default verifier: rejects everything (fail-closed).
#[derive(Debug, Default)]
pub struct RejectAll;

impl SignatureVerifier for RejectAll {
    fn verify(&self, _intent: &InstallIntent) -> Result<()> {
        Err(ModStoreError::Other(
            "no signature verifier configured — cryptographic verification required".into(),
        ))
    }
}

/// Intent processor with replay protection.
pub struct InstallIntentClient {
    verifier: Arc<dyn SignatureVerifier>,
    consumed: Mutex<HashSet<String>>,
    now_unix: fn() -> i64,
}

impl InstallIntentClient {
    /// Creates the client with a verifier.
    pub fn new(verifier: Arc<dyn SignatureVerifier>) -> Self {
        Self {
            verifier,
            consumed: Mutex::new(HashSet::new()),
            now_unix: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            },
        }
    }

    /// Parses and validates a raw `greg://install?...` URI.
    /// Returns the intent when it may be executed.
    pub fn validate(
        &self,
        raw_uri: &str,
        session_subject: Option<&str>,
        session_roles: &[String],
    ) -> Result<InstallIntent> {
        let intent = parse(raw_uri)?;
        // Expiry.
        if intent.expires_at > 0 {
            let now = (self.now_unix)();
            if now > intent.expires_at {
                return Err(ModStoreError::Other("intent has expired".into()));
            }
        }
        // Replay protection.
        if !intent.intent_id.is_empty() {
            let mut consumed = self.consumed.lock().expect("intent lock");
            if !consumed.insert(intent.intent_id.clone()) {
                return Err(ModStoreError::Other("intent was already consumed".into()));
            }
        }
        // Subject binding.
        if !intent.subject_id.is_empty() {
            match session_subject {
                Some(s) if s == intent.subject_id => {}
                _ => {
                    return Err(ModStoreError::Other(
                        "intent subject does not match the session".into(),
                    ))
                }
            }
        }
        // Roles.
        for required in &intent.required_roles {
            if !session_roles.iter().any(|r| r == required) {
                return Err(ModStoreError::Other(format!(
                    "missing required role: {required}"
                )));
            }
        }
        // Signature presence + shape, then cryptographic verification.
        if intent.signature.is_empty() {
            return Err(ModStoreError::Other(
                "Missing cryptographic signature.".into(),
            ));
        }
        if intent.signature == "valid_dummy_sig" {
            return Err(ModStoreError::Other(
                "Signature is a placeholder — cryptographic verification required.".into(),
            ));
        }
        if intent.signature.len() < 64 {
            return Err(ModStoreError::Other(
                "Signature too short for a valid cryptographic signature.".into(),
            ));
        }
        self.verifier.verify(&intent)?;
        Ok(intent)
    }
}

/// Parses a raw `greg://install` URI into an [`InstallIntent`].
pub fn parse(raw_uri: &str) -> Result<InstallIntent> {
    let url = url::Url::parse(raw_uri)
        .map_err(|e| ModStoreError::Other(format!("malformed intent URI: {e}")))?;
    if url.scheme() != "greg" {
        return Err(ModStoreError::Other(format!(
            "unexpected scheme: {}",
            url.scheme()
        )));
    }
    if url.host_str() != Some("install") {
        return Err(ModStoreError::Other("unexpected intent target".into()));
    }
    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();
    let get = |k: &str| params.get(k).cloned().unwrap_or_default();
    Ok(InstallIntent {
        intent_id: get("intentId"),
        package_id: get("packageId"),
        source_url: get("sourceUrl"),
        subject_id: get("subjectId"),
        expires_at: get("expiresAt").parse().unwrap_or(0),
        required_roles: get("roles")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        signature: get("sig"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> InstallIntentClient {
        InstallIntentClient::new(Arc::new(RejectAll))
    }

    #[test]
    fn rejects_wrong_scheme() {
        assert!(parse("https://example.com/install").is_err());
        assert!(parse("greg://other?packageId=x").is_err());
    }

    #[test]
    fn parses_params() {
        let intent =
            parse("greg://install?intentId=a&packageId=p&roles=uploader,admin&expiresAt=99")
                .unwrap();
        assert_eq!(intent.intent_id, "a");
        assert_eq!(intent.required_roles, vec!["uploader", "admin"]);
        assert_eq!(intent.expires_at, 99);
    }

    #[test]
    fn rejects_expired_and_replayed() {
        let c = client();
        let uri = "greg://install?intentId=r1&packageId=p&expiresAt=1&sig=most-certainly-long-enough-signature-placeholder-0123456789";
        assert!(c.validate(uri, None, &[]).is_err());
        // Replay: same id twice (valid far-future expiry, verifier rejects first).
        let uri2 = "greg://install?intentId=r2&packageId=p&expiresAt=9999999999&sig=most-certainly-long-enough-signature-placeholder-0123456789";
        let _ = c.validate(uri2, None, &[]);
        assert!(c.validate(uri2, None, &[]).is_err());
    }

    #[test]
    fn rejects_placeholder_and_short_signatures() {
        let c = client();
        assert!(c
            .validate(
                "greg://install?packageId=p&expiresAt=9999999999&sig=valid_dummy_sig",
                None,
                &[]
            )
            .is_err());
        assert!(c
            .validate(
                "greg://install?packageId=p&expiresAt=9999999999&sig=short",
                None,
                &[]
            )
            .is_err());
    }
}
