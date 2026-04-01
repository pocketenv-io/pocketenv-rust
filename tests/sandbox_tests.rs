use pocketenv::PocketenvClient;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sandbox_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "my-sandbox",
        "provider": "cloudflare",
        "status": "RUNNING",
        "description": "A test sandbox",
        "displayName": "My Sandbox",
        "baseSandbox": "openclaw",
        "uri": format!("at://did:plc:test/io.pocketenv.sandbox/{id}"),
        "topics": ["rust", "test"],
        "logo": null,
        "readme": null,
        "installs": 0,
        "repo": "https://github.com/example/repo",
        "vcpus": 2,
        "memory": 4,
        "disk": 10,
        "createdAt": "2024-01-01T00:00:00.000Z",
        "startedAt": "2024-01-01T00:01:00.000Z"
    })
}

// ── SandboxClient ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn sandbox_create() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.createSandbox"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sandbox_json("sb-1")))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client
        .sandboxes
        .create(pocketenv::CreateOptions::default())
        .await
        .unwrap();

    assert_eq!(sandbox.id, "sb-1");
    assert_eq!(sandbox.name.as_deref(), Some("my-sandbox"));
    assert_eq!(sandbox.status.as_deref(), Some("RUNNING"));
    assert_eq!(sandbox.vcpus, Some(2));
}

#[tokio::test]
async fn sandbox_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "sandbox": sandbox_json("sb-1") })),
        )
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    assert_eq!(sandbox.id, "sb-1");
    assert_eq!(sandbox.provider.as_deref(), Some("cloudflare"));
}

#[tokio::test]
async fn sandbox_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandboxes"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandboxes": [sandbox_json("sb-1"), sandbox_json("sb-2")],
            "total": 2
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandboxes = client.sandboxes.list(0, 10).await.unwrap();
    assert_eq!(sandboxes.len(), 2);
    assert_eq!(sandboxes[0].id, "sb-1");
    assert_eq!(sandboxes[1].id, "sb-2");
}

#[tokio::test]
async fn sandbox_list_by_actor() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.actor.getActorSandboxes"))
        .and(query_param("did", "did:plc:alice"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandboxes": [sandbox_json("sb-alice")],
            "total": 1
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandboxes = client
        .sandboxes
        .list_by_actor("did:plc:alice", 0, 5)
        .await
        .unwrap();
    assert_eq!(sandboxes.len(), 1);
    assert_eq!(sandboxes[0].id, "sb-alice");
}

#[tokio::test]
async fn sandbox_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.deleteSandbox"))
        .and(query_param("id", "sb-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.sandboxes.delete("sb-1").await.unwrap();
}

#[tokio::test]
async fn sandbox_claim() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.claimSandbox"))
        .and(query_param("id", "sb-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.claim("sb-1").await.unwrap();
    assert_eq!(sandbox.id, "sb-1");
}

// ── Sandbox handle ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn sandbox_start() {
    let server = MockServer::start().await;
    // First get the sandbox
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.startSandbox"))
        .and(query_param("id", "sb-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    sandbox.start().await.unwrap();
}

#[tokio::test]
async fn sandbox_stop() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.stopSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    sandbox.stop().await.unwrap();
}

#[tokio::test]
async fn sandbox_exec() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.exec"))
        .and(query_param("id", "sb-1"))
        .and(body_json(serde_json::json!({ "command": "echo hello" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "stdout": "hello\n",
            "stderr": "",
            "exitCode": 0
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    let result = sandbox.exec("echo hello").await.unwrap();
    assert_eq!(result.stdout, "hello\n");
    assert_eq!(result.exit_code, 0);
}

#[tokio::test]
async fn sandbox_expose_port() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.exposePort"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "previewUrl": "https://3000.sb-1.pocketenv.io"
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    let url = sandbox.expose(3000, Some("web server")).await.unwrap();
    assert_eq!(url.as_deref(), Some("https://3000.sb-1.pocketenv.io"));
}

#[tokio::test]
async fn sandbox_unexpose_port() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.unexposePort"))
        .and(query_param("id", "sb-1"))
        .and(body_json(serde_json::json!({ "port": 3000 })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    sandbox.unexpose(3000).await.unwrap();
}

#[tokio::test]
async fn sandbox_get_exposed_ports() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getExposedPorts"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ports": [
                { "port": 3000, "description": "web", "previewUrl": "https://3000.sb-1.pocketenv.io" },
                { "port": 8080, "description": null, "previewUrl": null }
            ]
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    let ports = sandbox.get_exposed_ports().await.unwrap();
    assert_eq!(ports.len(), 2);
    assert_eq!(ports[0].port, 3000);
    assert_eq!(
        ports[0].preview_url.as_deref(),
        Some("https://3000.sb-1.pocketenv.io")
    );
    assert_eq!(ports[1].port, 8080);
    assert!(ports[1].preview_url.is_none());
}

#[tokio::test]
async fn sandbox_expose_vscode() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.exposeVscode"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "previewUrl": "https://vscode.sb-1.pocketenv.io"
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    let url = sandbox.expose_vscode().await.unwrap();
    assert_eq!(url.as_deref(), Some("https://vscode.sb-1.pocketenv.io"));
}

#[tokio::test]
async fn sandbox_ssh_keys() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sandbox": sandbox_json("sb-1")
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.sandbox.putSshKeys"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSshKeys"))
        .and(query_param("id", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "key-1",
            "privateKey": "-----BEGIN RSA PRIVATE KEY-----",
            "publicKey": "ssh-rsa AAAA...",
            "createdAt": "2024-01-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let sandbox = client.sandboxes.get("sb-1").await.unwrap();
    sandbox
        .put_ssh_keys("-----BEGIN RSA PRIVATE KEY-----", "ssh-rsa AAAA...", "ssh-rsa AA...")
        .await
        .unwrap();
    let keys = sandbox.get_ssh_keys().await.unwrap();
    assert_eq!(keys.id, "key-1");
    assert_eq!(keys.public_key, "ssh-rsa AAAA...");
}

// ── Error handling ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn sandbox_get_not_found_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.sandbox.getSandbox"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    assert!(client.sandboxes.get("nonexistent").await.is_err());
}
