use pocketenv::PocketenvClient;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn file_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "path": "/etc/config.toml",
        "createdAt": "2024-01-01T00:00:00.000Z"
    })
}

#[tokio::test]
async fn file_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.file.addFile"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "file": {
                "sandboxId": "sb-1",
                "path": "/etc/config.toml",
                "content": "encrypted-content"
            }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .files
        .add("sb-1", "/etc/config.toml", "encrypted-content")
        .await
        .unwrap();
}

#[tokio::test]
async fn file_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.file.deleteFile"))
        .and(query_param("id", "file-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.files.delete("file-1").await.unwrap();
}

#[tokio::test]
async fn file_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.file.getFile"))
        .and(query_param("id", "file-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "file": file_json("file-1") })),
        )
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let file = client.files.get("file-1").await.unwrap();
    assert_eq!(file.id, "file-1");
    assert_eq!(file.path, "/etc/config.toml");
}

#[tokio::test]
async fn file_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.file.getFiles"))
        .and(query_param("sandboxId", "sb-1"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "30"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "files": [file_json("file-1"), file_json("file-2")],
            "total": 2
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let files = client.files.list("sb-1", 0, 30).await.unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].id, "file-1");
}

#[tokio::test]
async fn file_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.file.updateFile"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "id": "file-1",
            "file": { "path": "/etc/new.toml", "content": "new-content" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .files
        .update("file-1", "/etc/new.toml", "new-content")
        .await
        .unwrap();
}
