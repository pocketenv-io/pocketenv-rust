use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::{ClientInner, DEFAULT_API_URL};

const DEFAULT_BASE: &str = "at://did:plc:aturpi2ls3yvsmhc6wybomun/io.pocketenv.sandbox/openclaw";

// ── Public sandbox client ─────────────────────────────────────────────────────

/// Client for sandbox operations.
#[derive(Clone)]
pub struct SandboxClient {
    pub(crate) inner: Arc<ClientInner>,
}

impl SandboxClient {
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
        self.inner.url(&format!("io.pocketenv.sandbox.{}", method))
    }

    fn url_with_id(&self, method: &str, id: &str) -> reqwest::Url {
        let mut url = self.url(method);
        url.query_pairs_mut().append_pair("id", id);
        url
    }

    /// Create a new sandbox and return a handle to it.
    pub async fn create(&self, opts: CreateOptions) -> Result<Sandbox> {
        let body = CreateBody {
            base: opts.base.unwrap_or_else(|| DEFAULT_BASE.to_string()),
            name: opts.name,
            provider: opts.provider,
            repo: opts.repo,
            description: opts.description,
            vcpus: opts.vcpus,
            memory: opts.memory,
            disk: opts.disk,
            keep_alive: opts.keep_alive,
        };

        let view = self
            .inner
            .http
            .post(self.url("createSandbox"))
            .header("Authorization", self.inner.auth())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<SandboxView>()
            .await?;

        Ok(Sandbox::from_view(view, Arc::clone(&self.inner)))
    }

    /// Fetch a sandbox by ID, name, or URI.
    pub async fn get(&self, id: &str) -> Result<Sandbox> {
        #[derive(Deserialize)]
        struct GetResponse {
            sandbox: SandboxView,
        }

        let res = self
            .inner
            .http
            .get(self.url_with_id("getSandbox", id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<GetResponse>()
            .await?;

        Ok(Sandbox::from_view(res.sandbox, Arc::clone(&self.inner)))
    }

    /// List sandboxes (paginated).
    pub async fn list(&self, offset: u32, limit: u32) -> Result<Vec<Sandbox>> {
        #[derive(Deserialize)]
        struct ListResponse {
            sandboxes: Vec<SandboxView>,
        }

        let mut url = self.url("getSandboxes");
        url.query_pairs_mut()
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
            .json::<ListResponse>()
            .await?;

        Ok(res
            .sandboxes
            .into_iter()
            .map(|v| Sandbox::from_view(v, Arc::clone(&self.inner)))
            .collect())
    }

    /// List sandboxes for a specific actor (DID), paginated.
    pub async fn list_by_actor(&self, did: &str, offset: u32, limit: u32) -> Result<Vec<Sandbox>> {
        #[derive(Deserialize)]
        struct ListResponse {
            sandboxes: Vec<SandboxView>,
        }

        let mut url = self.inner.url("io.pocketenv.actor.getActorSandboxes");
        url.query_pairs_mut()
            .append_pair("did", did)
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
            .json::<ListResponse>()
            .await?;

        Ok(res
            .sandboxes
            .into_iter()
            .map(|v| Sandbox::from_view(v, Arc::clone(&self.inner)))
            .collect())
    }

    /// Delete a sandbox by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.inner
            .http
            .post(self.url_with_id("deleteSandbox", id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Claim an anonymous sandbox and associate it with the authenticated user.
    pub async fn claim(&self, id: &str) -> Result<Sandbox> {
        #[derive(Deserialize)]
        struct ClaimResponse {
            sandbox: SandboxView,
        }

        let res = self
            .inner
            .http
            .post(self.url_with_id("claimSandbox", id))
            .header("Authorization", self.inner.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<ClaimResponse>()
            .await?;

        Ok(Sandbox::from_view(res.sandbox, Arc::clone(&self.inner)))
    }
}

// ── Options / result types ───────────────────────────────────────────────────

/// Options for creating a sandbox.
#[derive(Debug, Clone, Default)]
pub struct CreateOptions {
    /// AT Protocol URI of the base template (defaults to openclaw).
    pub base: Option<String>,
    pub name: Option<String>,
    /// `"cloudflare"` | `"daytona"` | `"vercel"` | `"deno"` | `"sprites"`.
    pub provider: Option<String>,
    pub repo: Option<String>,
    pub description: Option<String>,
    pub vcpus: Option<u32>,
    /// Memory in GB.
    pub memory: Option<u32>,
    /// Disk in GB.
    pub disk: Option<u32>,
    pub keep_alive: Option<bool>,
}

/// Builder for creating a sandbox without a pre-existing `PocketenvClient`.
///
/// ```no_run
/// # #[tokio::main] async fn main() -> anyhow::Result<()> {
/// use pocketenv::Sandbox;
/// let sandbox = Sandbox::builder("my-env")
///     .provider("cloudflare")
///     .vcpus(2)
///     .memory(4)
///     .disk(10)
///     .token(std::env::var("POCKETENV_TOKEN")?)
///     .create()
///     .await?;
/// # Ok(()) }
/// ```
pub struct SandboxBuilder {
    name: String,
    api_url: String,
    token: Option<String>,
    provider: Option<String>,
    base: Option<String>,
    repo: Option<String>,
    description: Option<String>,
    vcpus: Option<u32>,
    memory: Option<u32>,
    disk: Option<u32>,
    keep_alive: Option<bool>,
}

impl SandboxBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            api_url: DEFAULT_API_URL.to_string(),
            token: None,
            provider: None,
            base: None,
            repo: None,
            description: None,
            vcpus: None,
            memory: None,
            disk: None,
            keep_alive: None,
        }
    }

    /// Override the API base URL (defaults to `https://api.pocketenv.io`).
    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    pub fn repo(mut self, repo: impl Into<String>) -> Self {
        self.repo = Some(repo.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn vcpus(mut self, vcpus: u32) -> Self {
        self.vcpus = Some(vcpus);
        self
    }

    /// Memory in GB.
    pub fn memory(mut self, memory: u32) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Disk in GB.
    pub fn disk(mut self, disk: u32) -> Self {
        self.disk = Some(disk);
        self
    }

    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    /// Create the sandbox. Requires `.token()` to have been called.
    pub async fn create(self) -> Result<Sandbox> {
        let token = self.token.ok_or_else(|| {
            anyhow::anyhow!("token is required — call .token(\"...\") on the builder")
        })?;
        let client = SandboxClient::new(self.api_url, token);
        client
            .create(CreateOptions {
                base: self.base,
                name: Some(self.name),
                provider: self.provider,
                repo: self.repo,
                description: self.description,
                vcpus: self.vcpus,
                memory: self.memory,
                disk: self.disk,
                keep_alive: self.keep_alive,
            })
            .await
    }
}

/// Result of executing a command inside a sandbox.
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// An exposed port with its public preview URL.
#[derive(Debug, Clone)]
pub struct Port {
    pub port: u16,
    pub description: Option<String>,
    pub preview_url: Option<String>,
}

/// SSH key pair stored on a sandbox.
#[derive(Debug, Clone)]
pub struct SshKeys {
    pub id: String,
    pub private_key: String,
    pub public_key: String,
    pub created_at: String,
}

// ── Internal serde types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SandboxView {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) base_sandbox: Option<String>,
    pub(crate) uri: Option<String>,
    pub(crate) topics: Option<Vec<String>>,
    pub(crate) logo: Option<String>,
    pub(crate) readme: Option<String>,
    pub(crate) installs: Option<u32>,
    pub(crate) repo: Option<String>,
    pub(crate) vcpus: Option<u32>,
    pub(crate) memory: Option<u32>,
    pub(crate) disk: Option<u32>,
    pub(crate) created_at: Option<String>,
    pub(crate) started_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateBody {
    base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vcpus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecResponse {
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
    #[serde(default)]
    exit_code: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExposePortResponse {
    preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortView {
    port: u16,
    description: Option<String>,
    preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshKeysView {
    id: String,
    private_key: String,
    public_key: String,
    created_at: String,
}

// ── Sandbox handle ───────────────────────────────────────────────────────────

/// A handle to a sandbox. Holds its metadata and provides methods for
/// start / stop / exec / expose operations.
#[derive(Debug, Clone)]
pub struct Sandbox {
    pub id: String,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub base_sandbox: Option<String>,
    pub uri: Option<String>,
    pub topics: Option<Vec<String>>,
    pub logo: Option<String>,
    pub readme: Option<String>,
    pub installs: Option<u32>,
    pub repo: Option<String>,
    pub vcpus: Option<u32>,
    pub memory: Option<u32>,
    pub disk: Option<u32>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    client: Arc<ClientInner>,
}

impl Sandbox {
    pub fn builder(name: impl Into<String>) -> SandboxBuilder {
        SandboxBuilder::new(name)
    }

    pub(crate) fn from_view(v: SandboxView, client: Arc<ClientInner>) -> Self {
        Self {
            id: v.id,
            name: v.name,
            provider: v.provider,
            status: v.status,
            description: v.description,
            display_name: v.display_name,
            base_sandbox: v.base_sandbox,
            uri: v.uri,
            topics: v.topics,
            logo: v.logo,
            readme: v.readme,
            installs: v.installs,
            repo: v.repo,
            vcpus: v.vcpus,
            memory: v.memory,
            disk: v.disk,
            created_at: v.created_at,
            started_at: v.started_at,
            client,
        }
    }

    fn url(&self, method: &str) -> reqwest::Url {
        self.client.url(&format!("io.pocketenv.sandbox.{}", method))
    }

    fn url_with_id(&self, method: &str) -> reqwest::Url {
        let mut url = self.url(method);
        url.query_pairs_mut().append_pair("id", &self.id);
        url
    }

    /// Start the sandbox.
    pub async fn start(&self) -> Result<()> {
        self.client
            .http
            .post(self.url_with_id("startSandbox"))
            .header("Authorization", self.client.auth())
            .json(&serde_json::json!({}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Stop the sandbox.
    pub async fn stop(&self) -> Result<()> {
        self.client
            .http
            .post(self.url_with_id("stopSandbox"))
            .header("Authorization", self.client.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Delete the sandbox.
    pub async fn delete(&self) -> Result<()> {
        self.client
            .http
            .post(self.url_with_id("deleteSandbox"))
            .header("Authorization", self.client.auth())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Execute a shell command inside the sandbox and return its output.
    pub async fn exec(&self, command: &str) -> Result<ExecResult> {
        let res = self
            .client
            .http
            .post(self.url_with_id("exec"))
            .header("Authorization", self.client.auth())
            .json(&serde_json::json!({ "command": command }))
            .send()
            .await?
            .error_for_status()?
            .json::<ExecResponse>()
            .await?;

        Ok(ExecResult {
            stdout: res.stdout,
            stderr: res.stderr,
            exit_code: res.exit_code,
        })
    }

    /// Expose a port and return the public preview URL (if any).
    pub async fn expose(&self, port: u16, description: Option<&str>) -> Result<Option<String>> {
        let mut body = serde_json::json!({ "port": port });
        if let Some(desc) = description {
            body["description"] = serde_json::Value::String(desc.to_string());
        }

        let res = self
            .client
            .http
            .post(self.url_with_id("exposePort"))
            .header("Authorization", self.client.auth())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<ExposePortResponse>()
            .await?;

        Ok(res.preview_url)
    }

    /// Unexpose a previously exposed port.
    pub async fn unexpose(&self, port: u16) -> Result<()> {
        self.client
            .http
            .post(self.url_with_id("unexposePort"))
            .header("Authorization", self.client.auth())
            .json(&serde_json::json!({ "port": port }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// List all currently exposed ports.
    pub async fn get_exposed_ports(&self) -> Result<Vec<Port>> {
        #[derive(Deserialize)]
        struct Response {
            ports: Vec<PortView>,
        }

        let res = self
            .client
            .http
            .get(self.url_with_id("getExposedPorts"))
            .header("Authorization", self.client.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;

        Ok(res
            .ports
            .into_iter()
            .map(|p| Port {
                port: p.port,
                description: p.description,
                preview_url: p.preview_url,
            })
            .collect())
    }

    /// Expose VS Code server and return the browser URL.
    pub async fn expose_vscode(&self) -> Result<Option<String>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            preview_url: Option<String>,
        }

        let res = self
            .client
            .http
            .post(self.url_with_id("exposeVscode"))
            .header("Authorization", self.client.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;

        Ok(res.preview_url)
    }

    /// Store SSH keys on the sandbox.
    pub async fn put_ssh_keys(
        &self,
        private_key: &str,
        public_key: &str,
        redacted: &str,
    ) -> Result<()> {
        self.client
            .http
            .post(self.url("putSshKeys"))
            .header("Authorization", self.client.auth())
            .json(&serde_json::json!({
                "id": self.id,
                "privateKey": private_key,
                "publicKey": public_key,
                "redacted": redacted,
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Retrieve the SSH keys stored on the sandbox.
    pub async fn get_ssh_keys(&self) -> Result<SshKeys> {
        let view = self
            .client
            .http
            .get(self.url_with_id("getSshKeys"))
            .header("Authorization", self.client.auth())
            .send()
            .await?
            .error_for_status()?
            .json::<SshKeysView>()
            .await?;

        Ok(SshKeys {
            id: view.id,
            private_key: view.private_key,
            public_key: view.public_key,
            created_at: view.created_at,
        })
    }
}
