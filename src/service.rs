use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ClientInner;

// ── Public types ─────────────────────────────────────────────────────────────

/// The running status of a service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Unknown(String),
}

impl<'de> Deserialize<'de> for ServiceStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.as_str() {
            "RUNNING" => ServiceStatus::Running,
            "STOPPED" => ServiceStatus::Stopped,
            other => ServiceStatus::Unknown(other.to_string()),
        })
    }
}

/// A background service managed inside a sandbox.
#[derive(Debug, Clone)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub command: String,
    pub ports: Option<Vec<u16>>,
    pub description: Option<String>,
    pub status: ServiceStatus,
    pub created_at: String,
}

/// Options for creating or updating a service.
#[derive(Debug, Clone, Default)]
pub struct ServiceOptions {
    pub ports: Option<Vec<u16>>,
    pub description: Option<String>,
}

// ── Internal serde types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceView {
    id: String,
    name: String,
    command: String,
    ports: Option<Vec<u16>>,
    description: Option<String>,
    status: ServiceStatus,
    created_at: String,
}

impl From<ServiceView> for Service {
    fn from(v: ServiceView) -> Self {
        Self {
            id: v.id,
            name: v.name,
            command: v.command,
            ports: v.ports,
            description: v.description,
            status: v.status,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceInput<'a> {
    name: &'a str,
    command: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    ports: Option<&'a [u16]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Client for service operations.
#[derive(Clone)]
pub struct ServiceClient {
    inner: Arc<ClientInner>,
}

impl ServiceClient {
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
            .url(&format!("io.pocketenv.service.{}", method))
    }

    fn url_with_sandbox(&self, method: &str, sandbox_id: &str) -> reqwest::Url {
        let mut url = self.url(method);
        url.query_pairs_mut().append_pair("sandboxId", sandbox_id);
        url
    }

    fn url_with_service(&self, method: &str, service_id: &str) -> reqwest::Url {
        let mut url = self.url(method);
        url.query_pairs_mut().append_pair("serviceId", service_id);
        url
    }

    /// Create a service inside a sandbox.
    pub async fn add(
        &self,
        sandbox_id: &str,
        name: &str,
        command: &str,
        opts: ServiceOptions,
    ) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_sandbox("addService", sandbox_id))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "service": ServiceInput {
                    name,
                    command,
                    ports: opts.ports.as_deref(),
                    description: opts.description.as_deref(),
                }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Delete a service by ID.
    pub async fn delete(&self, service_id: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_service("deleteService", service_id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// List services for a sandbox.
    pub async fn list(&self, sandbox_id: &str) -> Result<Vec<Service>> {
        #[derive(Deserialize)]
        struct Response {
            services: Vec<ServiceView>,
        }
        let res = self
            .inner
            .http
            .get(self.url_with_sandbox("getServices", sandbox_id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;
        Ok(res.services.into_iter().map(Into::into).collect())
    }

    /// Start a service.
    pub async fn start(&self, service_id: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_service("startService", service_id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Stop a service.
    pub async fn stop(&self, service_id: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_service("stopService", service_id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Restart a service.
    pub async fn restart(&self, service_id: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_service("restartService", service_id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Update a service's configuration.
    pub async fn update(
        &self,
        service_id: &str,
        name: &str,
        command: &str,
        opts: ServiceOptions,
    ) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_service("updateService", service_id))
            .header("Authorization", self.inner.auth())
            .json(&serde_json::json!({
                "service": ServiceInput {
                    name,
                    command,
                    ports: opts.ports.as_deref(),
                    description: opts.description.as_deref(),
                }
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
