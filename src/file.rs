use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ClientInner;

// ── Public types ─────────────────────────────────────────────────────────────

/// A file injected into a sandbox at a specific path.
#[derive(Debug, Clone)]
pub struct File {
    pub id: String,
    pub path: String,
    pub created_at: String,
}

// ── Internal serde types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileView {
    id: String,
    path: String,
    created_at: String,
}

impl From<FileView> for File {
    fn from(v: FileView) -> Self {
        Self {
            id: v.id,
            path: v.path,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileInput<'a> {
    sandbox_id: &'a str,
    path: &'a str,
    content: &'a str,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Client for file operations.
#[derive(Clone)]
pub struct FileClient {
    inner: Arc<ClientInner>,
}

impl FileClient {
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
            .url(&format!("io.pocketenv.file.{}", method))
    }

    /// Add (upload) a file to a sandbox.
    ///
    /// `content` should be encrypted by the caller before sending.
    pub async fn add(&self, sandbox_id: &str, path: &str, content: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url("addFile"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "file": FileInput { sandbox_id, path, content }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Delete a file by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut url = self.url("deleteFile");
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

    /// Fetch a single file record by ID.
    pub async fn get(&self, id: &str) -> Result<File> {
        #[derive(Deserialize)]
        struct Response {
            file: FileView,
        }
        let mut url = self.url("getFile");
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
        Ok(res.file.into())
    }

    /// List files for a sandbox (paginated).
    pub async fn list(&self, sandbox_id: &str, offset: u32, limit: u32) -> Result<Vec<File>> {
        #[derive(Deserialize)]
        struct Response {
            files: Vec<FileView>,
        }
        let mut url = self.url("getFiles");
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
        Ok(res.files.into_iter().map(Into::into).collect())
    }

    /// Update the path and/or content of an existing file.
    pub async fn update(&self, id: &str, path: &str, content: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url("updateFile"))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "id": id,
                "file": { "path": path, "content": content }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
