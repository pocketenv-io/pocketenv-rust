use pocketenv::PocketenvClient;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn volume_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "data-vol",
        "path": "/data",
        "createdAt": "2024-01-01T00:00:00.000Z"
    })
}

#[tokio::test]
async fn volume_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.volume.addVolume"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "volume": { "sandboxId": "sb-1", "name": "data-vol", "path": "/data" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .volumes
        .add("sb-1", "data-vol", "/data")
        .await
        .unwrap();
}

#[tokio::test]
async fn volume_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.volume.deleteVolume"))
        .and(query_param("id", "vol-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.volumes.delete("vol-1").await.unwrap();
}

#[tokio::test]
async fn volume_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.volume.getVolume"))
        .and(query_param("id", "vol-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "volume": volume_json("vol-1") })),
        )
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let vol = client.volumes.get("vol-1").await.unwrap();
    assert_eq!(vol.id, "vol-1");
    assert_eq!(vol.path, "/data");
}

#[tokio::test]
async fn volume_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.volume.getVolumes"))
        .and(query_param("sandboxId", "sb-1"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "30"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "volumes": [volume_json("vol-1")],
            "total": 1
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let vols = client.volumes.list("sb-1", 0, 30).await.unwrap();
    assert_eq!(vols.len(), 1);
    assert_eq!(vols[0].name, "data-vol");
}

#[tokio::test]
async fn volume_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.volume.updateVolume"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "id": "vol-1",
            "volume": { "sandboxId": "sb-1", "name": "data-vol", "path": "/mnt/data" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .volumes
        .update("vol-1", "sb-1", "data-vol", "/mnt/data")
        .await
        .unwrap();
}
