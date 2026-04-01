use pocketenv::PocketenvClient;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn variable_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": "MY_VAR",
        "value": "hello",
        "createdAt": "2024-01-01T00:00:00.000Z"
    })
}

#[tokio::test]
async fn variable_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.variable.addVariable"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "variable": { "sandboxId": "sb-1", "name": "MY_VAR", "value": "hello" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .variables
        .add("sb-1", "MY_VAR", "hello")
        .await
        .unwrap();
}

#[tokio::test]
async fn variable_delete() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.variable.deleteVariable"))
        .and(query_param("id", "var-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client.variables.delete("var-1").await.unwrap();
}

#[tokio::test]
async fn variable_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.variable.getVariable"))
        .and(query_param("id", "var-1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "variable": variable_json("var-1") })),
        )
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let var = client.variables.get("var-1").await.unwrap();
    assert_eq!(var.id, "var-1");
    assert_eq!(var.name, "MY_VAR");
    assert_eq!(var.value, "hello");
}

#[tokio::test]
async fn variable_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/xrpc/io.pocketenv.variable.getVariables"))
        .and(query_param("sandboxId", "sb-1"))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "30"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "variables": [variable_json("var-1"), variable_json("var-2")],
            "total": 2
        })))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    let vars = client.variables.list("sb-1", 0, 30).await.unwrap();
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].id, "var-1");
}

#[tokio::test]
async fn variable_update() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/xrpc/io.pocketenv.variable.updateVariable"))
        .and(header("Authorization", "Bearer test-token"))
        .and(body_json(serde_json::json!({
            "id": "var-1",
            "variable": { "sandboxId": "sb-1", "name": "MY_VAR", "value": "world" }
        })))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = PocketenvClient::new(server.uri(), "test-token");
    client
        .variables
        .update("var-1", "sb-1", "MY_VAR", "world")
        .await
        .unwrap();
}
