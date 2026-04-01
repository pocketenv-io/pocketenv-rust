use pocketenv::{PocketenvClient, ServiceOptions, ServiceStatus};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn service_json(id: &str, status: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "web",
        "command": "npm start",
        "ports": [3000],
        "description": "Web server",
        "status": status,
        "createdAt": "2024-01-01T00:00:00.000Z"
    })
}

#[tokio::test]
async fn service_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.service.addService"))
        .and(query_param("sandboxId", "sb-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .services
        .add(
            "sb-1",
            "web",
            "npm start",
            ServiceOptions {
                ports: Some(vec![3000]),
                description: Some("Web server".to_string()),
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn service_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.service.deleteService"))
        .and(query_param("serviceId", "svc-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.services.delete("svc-1").await.unwrap();
}

#[tokio::test]
async fn service_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.service.getServices"))
        .and(query_param("sandboxId", "sb-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "services": [
                service_json("svc-1", "RUNNING"),
                service_json("svc-2", "STOPPED")
            ]
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let services = client.services.list("sb-1").await.unwrap();
    assert_eq!(services.len(), 2);
    assert_eq!(services[0].id, "svc-1");
    assert_eq!(services[0].status, ServiceStatus::Running);
    assert_eq!(services[1].status, ServiceStatus::Stopped);
}

#[tokio::test]
async fn service_start() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.service.startService"))
        .and(query_param("serviceId", "svc-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.services.start("svc-1").await.unwrap();
}

#[tokio::test]
async fn service_stop() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.service.stopService"))
        .and(query_param("serviceId", "svc-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.services.stop("svc-1").await.unwrap();
}

#[tokio::test]
async fn service_restart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.service.restartService"))
        .and(query_param("serviceId", "svc-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.services.restart("svc-1").await.unwrap();
}

#[tokio::test]
async fn service_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.service.updateService"))
        .and(query_param("serviceId", "svc-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .services
        .update(
            "svc-1",
            "web",
            "node server.js",
            ServiceOptions {
                ports: Some(vec![8080]),
                description: None,
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn service_status_unknown_variant() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.service.getServices"))
        .and(query_param("sandboxId", "sb-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "services": [service_json("svc-1", "STARTING")]
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let services = client.services.list("sb-1").await.unwrap();
    assert_eq!(
        services[0].status,
        ServiceStatus::Unknown("STARTING".to_string())
    );
}
