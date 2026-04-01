use pocketenv::PocketenvClient;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn secret_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "DB_PASSWORD",
        "createdAt": "2024-01-01T00:00:00.000Z"
    })
}

#[tokio::test]
async fn secret_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.secret.addSecret"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "secret": { "sandboxId": "sb-1", "name": "DB_PASSWORD", "value": "encrypted-blob" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .secrets
        .add("sb-1", "DB_PASSWORD", "encrypted-blob")
        .await
        .unwrap();
}

#[tokio::test]
async fn secret_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.secret.deleteSecret"))
        .and(query_param("id", "sec-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.secrets.delete("sec-1").await.unwrap();
}

#[tokio::test]
async fn secret_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.secret.getSecret"))
        .and(query_param("id", "sec-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "secret": secret_json("sec-1") })),
        )
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let secret = client.secrets.get("sec-1").await.unwrap();
    assert_eq!(secret.id, "sec-1");
    assert_eq!(secret.name, "DB_PASSWORD");
}

#[tokio::test]
async fn secret_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.secret.getSecrets"))
        .and(query_param("sandboxId", "sb-1"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "30"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "secrets": [secret_json("sec-1"), secret_json("sec-2")],
            "total": 2
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let secrets = client.secrets.list("sb-1", 0, 30).await.unwrap();
    assert_eq!(secrets.len(), 2);
    assert_eq!(secrets[1].id, "sec-2");
}

#[tokio::test]
async fn secret_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.secret.updateSecret"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "id": "sec-1",
            "secret": { "sandboxId": "sb-1", "name": "DB_PASSWORD", "value": "new-encrypted" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .secrets
        .update("sec-1", "sb-1", "DB_PASSWORD", "new-encrypted")
        .await
        .unwrap();
}
