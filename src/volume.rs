use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ClientInner;

// ── Public types ─────────────────────────────────────────────────────────────

/// A persistent volume mounted in a sandbox.
#[derive(Debug, Clone)]
pub struct Volume {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
}

// ── Internal serde types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VolumeView {
    id: String,
    name: String,
    path: String,
    created_at: String,
}

impl From<VolumeView> for Volume {
    fn from(v: VolumeView) -> Self {
        Self {
            id: v.id,
            name: v.name,
            path: v.path,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VolumeInput<'a> {
    sandbox_id: &'a str,
    name: &'a str,
    path: &'a str,
}

#[derive(Clone)]
pub struct VolumeClient {
    inner: Arc<ClientInner>,
}

impl VolumeClient {
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
        self.inner.url(&format!("io.pocketenv.volume.{}", method))
    }

    /// Add a volume to a sandbox.
    pub async fn add(&self, sandbox_id: &str, name: &str, path: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url("addVolume"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "volume": VolumeInput { sandbox_id, name, path }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut url = self.url("deleteVolume");
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

    pub async fn get(&self, id: &str) -> Result<Volume> {
        #[derive(Deserialize)]
        struct Response {
            volume: VolumeView,
        }
        let mut url = self.url("getVolume");
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
        Ok(res.volume.into())
    }

    pub async fn list(&self, sandbox_id: &str, offset: u32, limit: u32) -> Result<Vec<Volume>> {
        #[derive(Deserialize)]
        struct Response {
            volumes: Vec<VolumeView>,
        }
        let mut url = self.url("getVolumes");
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
        Ok(res.volumes.into_iter().map(Into::into).collect())
    }

    pub async fn update(&self, id: &str, sandbox_id: &str, name: &str, path: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url("updateVolume"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "id": id,
                "volume": VolumeInput { sandbox_id, name, path }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
