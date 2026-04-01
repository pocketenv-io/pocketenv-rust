use std::sync::Arc;

// ── Shared HTTP state ────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct ClientInner {
    pub(crate) api_url: String,
    pub(crate) token: String,
    pub(crate) http: reqwest::Client,
}

impl ClientInner {
    pub(crate) fn url(&self, action: &str) -> reqwest::Url {
        reqwest::Url::parse(&format!(
            "{}/xrpc/{}",
            self.api_url.trim_end_matches('/'),
            action
        ))
        .expect("valid base URL")
    }

    pub(crate) fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

// ── Top-level client ─────────────────────────────────────────────────────────

use crate::{
    file::FileClient, sandbox::SandboxClient, secret::SecretClient, service::ServiceClient,
    variable::VariableClient, volume::VolumeClient,
};

/// The main entry point for the Pocketenv SDK.
///
/// ```no_run
/// use pocketenv::PocketenvClient;
///
/// let client = PocketenvClient::new("https://api.pocketenv.io", "my-token");
/// ```
#[derive(Clone)]
pub struct PocketenvClient {
    pub sandboxes: SandboxClient,
    pub variables: VariableClient,
    pub secrets: SecretClient,
    pub volumes: VolumeClient,
    pub files: FileClient,
    pub services: ServiceClient,
}

impl PocketenvClient {
    pub fn new(api_url: impl Into<String>, token: impl Into<String>) -> Self {
        let inner = Arc::new(ClientInner {
            api_url: api_url.into(),
            token: token.into(),
            http: reqwest::Client::new(),
        });
        Self {
            sandboxes: SandboxClient::from_inner(Arc::clone(&inner)),
            variables: VariableClient::from_inner(Arc::clone(&inner)),
            secrets: SecretClient::from_inner(Arc::clone(&inner)),
            volumes: VolumeClient::from_inner(Arc::clone(&inner)),
            files: FileClient::from_inner(Arc::clone(&inner)),
            services: ServiceClient::from_inner(Arc::clone(&inner)),
        }
    }
}
