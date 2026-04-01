use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ClientInner;

// ── Public types ─────────────────────────────────────────────────────────────

/// An environment variable attached to a sandbox.
#[derive(Debug, Clone)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub value: String,
    pub created_at: String,
}

// ── Internal serde types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VariableView {
    id: String,
    name: String,
    value: String,
    created_at: String,
}

impl From<VariableView> for Variable {
    fn from(v: VariableView) -> Self {
        Self {
            id: v.id,
            name: v.name,
            value: v.value,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariableInput<'a> {
    sandbox_id: &'a str,
    name: &'a str,
    value: &'a str,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Client for environment variable operations.
#[derive(Clone)]
pub struct VariableClient {
    inner: Arc<ClientInner>,
}

impl VariableClient {
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
            .url(&format!("io.pocketenv.variable.{}", method))
    }

    /// Add an environment variable to a sandbox.
    pub async fn add(&self, sandbox_id: &str, name: &str, value: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url("addVariable"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "variable": VariableInput { sandbox_id, name, value }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Delete an environment variable by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut url = self.url("deleteVariable");
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

    /// Fetch a single environment variable by ID.
    pub async fn get(&self, id: &str) -> Result<Variable> {
        #[derive(Deserialize)]
        struct Response {
            variable: VariableView,
        }
        let mut url = self.url("getVariable");
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
        Ok(res.variable.into())
    }

    /// List environment variables for a sandbox (paginated).
    pub async fn list(
        &self,
        sandbox_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<Variable>> {
        #[derive(Deserialize)]
        struct Response {
            variables: Vec<VariableView>,
        }
        let mut url = self.url("getVariables");
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
        Ok(res.variables.into_iter().map(Into::into).collect())
    }

    /// Update an existing environment variable.
    pub async fn update(
        &self,
        id: &str,
        sandbox_id: &str,
        name: &str,
        value: &str,
    ) -> Result<()> {
        self.inner
            .http
            .post(self.url("updateVariable"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "id": id,
                "variable": VariableInput { sandbox_id, name, value }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
