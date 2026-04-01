# pocketenv

A Rust SDK for the [Pocketenv](https://pocketenv.io) API — manage cloud sandboxes, environment variables, secrets, files, volumes, and background services programmatically.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pocketenv = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use pocketenv::Sandbox;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a sandbox with the builder — defaults to https://api.pocketenv.io
    let sandbox = Sandbox::builder("my-sandbox")
        .provider("cloudflare")
        .vcpus(2)
        .memory(4)  // GB
        .disk(10)   // GB
        .token(std::env::var("POCKETENV_TOKEN")?)
        .create()
        .await?;

    println!("Created sandbox: {}", sandbox.id);

    // Start it and run a command
    sandbox.start().await?;
    let result = sandbox.exec("echo hello from pocketenv").await?;
    println!("stdout: {}", result.stdout);
    println!("exit code: {}", result.exit_code);

    // Expose a port
    let url = sandbox.expose(3000, Some("Web app")).await?;
    println!("Preview URL: {}", url.unwrap_or_default());

    // Stop and clean up
    sandbox.stop().await?;
    sandbox.delete().await?;

    Ok(())
}
```

## Usage

### Client Initialization

```rust
// Default base URL (https://api.pocketenv.io)
let client = PocketenvClient::with_token(token);

// Custom base URL
let client = PocketenvClient::new("https://api.pocketenv.io", token);
```

### Sandboxes

```rust
let sandbox = Sandbox::builder("dev")
    .provider("cloudflare")
    .vcpus(2)
    .memory(4)
    .disk(10)
    .token(token)
    .create()
    .await?;

// Get by ID, name, or AT URI
let sandbox = client.sandboxes.get("sb-abc123").await?;

// List (paginated)
let sandboxes = client.sandboxes.list(0, 20).await?;

// List by actor DID
let sandboxes = client.sandboxes.list_by_actor("did:plc:...", 0, 20).await?;

// Claim an anonymous sandbox
let sandbox = client.sandboxes.claim("sb-abc123").await?;

// Delete
client.sandboxes.delete("sb-abc123").await?;

// Lifecycle operations on a sandbox handle
sandbox.start().await?;
sandbox.stop().await?;
sandbox.delete().await?;

// Execute a shell command
let result = sandbox.exec("ls -la /workspace").await?;
// result.stdout, result.stderr, result.exit_code

// Port exposure
let port = sandbox.expose(8080, Some("API server".into())).await?;
// port.preview_url contains the public URL
sandbox.unexpose(8080).await?;
let ports = sandbox.get_exposed_ports().await?;

// Expose VS Code server
let port = sandbox.expose_vscode().await?;

// SSH keys
sandbox.put_ssh_keys(private_key, public_key, redacted).await?;
let keys = sandbox.get_ssh_keys().await?;
```

### Environment Variables

```rust
// Add
let var = client.variables.add("sb-abc123", "DATABASE_URL", "postgres://...").await?;

// List
let vars = client.variables.list("sb-abc123", 0, 50).await?;

// Get / Update / Delete
let var = client.variables.get(&var.id).await?;
client.variables.update(&var.id, "sb-abc123", "DATABASE_URL", "postgres://new").await?;
client.variables.delete(&var.id).await?;
```

### Secrets

```rust
// Add (value should be pre-encrypted)
let secret = client.secrets.add("sb-abc123", "API_KEY", encrypted_value).await?;

// List (names only — values are never returned)
let secrets = client.secrets.list("sb-abc123", 0, 50).await?;

// Get / Update / Delete
let secret = client.secrets.get(&secret.id).await?;
client.secrets.update(&secret.id, "sb-abc123", "API_KEY", new_encrypted_value).await?;
client.secrets.delete(&secret.id).await?;
```

### Files

```rust
// Inject a file into a sandbox
let file = client.files.add("sb-abc123", "/workspace/.env", content).await?;

// List / Get / Update / Delete
let files = client.files.list("sb-abc123", 0, 50).await?;
let file = client.files.get(&file.id).await?;
client.files.update(&file.id, "/workspace/.env", new_content).await?;
client.files.delete(&file.id).await?;
```

### Volumes

```rust
// Mount a persistent volume
let volume = client.volumes.add("sb-abc123", "data", "/workspace/data").await?;

// List / Get / Update / Delete
let volumes = client.volumes.list("sb-abc123", 0, 50).await?;
let volume = client.volumes.get(&volume.id).await?;
client.volumes.update(&volume.id, "sb-abc123", "data", "/data").await?;
client.volumes.delete(&volume.id).await?;
```

### Services

```rust
use pocketenv::service::ServiceOptions;

// Create a background service
let service = client.services.add(
    "sb-abc123",
    "web",
    "node server.js",
    ServiceOptions {
        ports: Some(vec![3000]),
        description: Some("Node.js web server".into()),
    },
).await?;

// List
let services = client.services.list("sb-abc123").await?;

// Lifecycle
client.services.start(&service.id).await?;
client.services.stop(&service.id).await?;
client.services.restart(&service.id).await?;

// Update / Delete
client.services.update(&service.id, "web", "node server.js", opts).await?;
client.services.delete(&service.id).await?;
```

## License

[MIT](LICENSE)
