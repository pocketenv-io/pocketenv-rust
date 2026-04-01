use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ClientInner;

// ── Public types ─────────────────────────────────────────────────────────────

/// A secret attached to a sandbox (value is encrypted at rest; only the name
/// is returned by the list / get endpoints).
#[derive(Debug, Clone)]
pub struct Secret {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

// ── Internal serde types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecretView {
    id: String,
    name: String,
    created_at: String,
}

impl From<SecretView> for Secret {
    fn from(v: SecretView) -> Self {
        Self {
            id: v.id,
            name: v.name,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SecretInput<'a> {
    sandbox_id: &'a str,
    name: &'a str,
    value: &'a str,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Client for secret operations.
#[derive(Clone)]
pub struct SecretClient {
    inner: Arc<ClientInner>,
}

impl SecretClient {
    pub fn new(api_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(ClientInner {
                api_url: api_url.into(),
                token: token.into(),
                http: reqwest::Client::new(),
            }),
        }
    }

    pub(crate) fn from_inner(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    fn url(&self, method: &str) -> reqwest::Url {
        self.inner
            .url(&format!("io.pocketenv.secret.{}", method))
    }

    /// Add a secret to a sandbox. The `value` should be encrypted by the
    /// caller before sending (e.g. with libsodium / `crypto_box`).
    pub async fn add(&self, sandbox_id: &str, name: &str, value: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url("addSecret"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "secret": SecretInput { sandbox_id, name, value }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Delete a secret by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut url = self.url("deleteSecret");
        url.query_pairs_mut().append_pair("id", id);
        self.inner
            .http
            .post(url)
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Fetch a single secret by ID.
    pub async fn get(&self, id: &str) -> Result<Secret> {
        #[derive(Deserialize)]
        struct Response {
            secret: SecretView,
        }
        let mut url = self.url("getSecret");
        url.query_pairs_mut().append_pair("id", id);
        let res = self
            .inner
            .http
            .get(url)
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;
        Ok(res.secret.into())
    }

    /// List secrets for a sandbox (paginated). Values are not returned.
    pub async fn list(
        &self,
        sandbox_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Secret>> {
        #[derive(Deserialize)]
        struct Response {
            secrets: Vec<SecretView>,
        }
        let mut url = self.url("getSecrets");
        url.query_pairs_mut()
            .append_pair("sandboxId", sandbox_id)
            .append_pair("offset", &offset.to_string())
            .append_pair("limit", &limit.to_string());
        let res = self
            .inner
            .http
            .get(url)
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;
        Ok(res.secrets.into_iter().map(Into::into).collect())
    }

    /// Update an existing secret.
    pub async fn update(
        &self,
        id: &str,
        sandbox_id: &str,
        name: &str,
        value: &str,
    ) -> Result<()> {
        self.inner
            .http
            .post(self.url("updateSecret"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "id": id,
                "secret": SecretInput { sandbox_id, name, value }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
