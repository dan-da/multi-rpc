# multi-rpc

[![Crates.io](https://img.shields.io/crates/v/multi-rpc.svg)](https://crates.io/crates/multi-rpc)
[![Docs.rs](https://docs.rs/multi-rpc/badge.svg)](https://docs.rs/multi-rpc)

Define your Rust service trait once, and serve it over multiple RPC and REST protocols simultaneously.

`multi-rpc` uses procedural macros to generate the necessary boilerplate for serving a single business logic implementation across different transport layers. This saves you from writing and maintaining protocol-specific adapter code.

## Supported Protocols

* **[tarpc](https://github.com/google/tarpc)**: A typed RPC framework for Rust.
* **REST**: A RESTful API server using **[Axum](https://github.com/tokio-rs/axum)**.
* **JSON-RPC**: A JSON-RPC 2.0 server using **[jsonrpsee](https://github.com/paritytech/jsonrpsee)**.

## Installation

Add `multi-rpc` to your dependencies and enable the features for the protocols you want to use.

```sh
cargo add multi-rpc -F tarpc -F rest-axum -F jsonrpsee
```

Or add it to your `Cargo.toml` manually:
```toml
[dependencies]
multi-rpc = { version = "0.1.0", features = ["tarpc", "rest-axum", "jsonrpsee"] }
```

## Usage Example

Here is a complete example of defining a `Greeter` service, running the servers, and calling its methods from three different clients.

### 1. Define and Implement the Service (Server-side)

Use the `#[multi_rpc_trait]` and `#[multi_rpc_impl]` attributes. The function signatures remain pure, protocol-agnostic Rust.

```rust
// In your library (e.g., src/lib.rs)
use multi_rpc::prelude::*;

// A custom newtype for all return values ensures consistent serialization.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MyResult(pub Result<String, RpcError>);

#[multi_rpc_trait]
pub trait Greeter {
    /// A simple method that takes a path parameter.
    async fn greet(&self, name: String) -> MyResult;

    /// A more complex method that mixes a path parameter and a multi-field JSON body.
    async fn update_settings(&self, user_id: u64, brightness: u32, theme: String) -> MyResult;
}

#[derive(Clone)]
pub struct MyGreeter(pub String);

#[multi_rpc_impl]
impl Greeter for MyGreeter {
    #[rest(method = GET, path = "/greet/{name}")]
    async fn greet(&self, name: String) -> MyResult {
        MyResult(Ok(format!("Hello, {}! My name is {}.", name, self.0)))
    }

    #[rest(method = POST, path = "/users/{user_id}/settings", body(brightness, theme))]
    async fn update_settings(&self, user_id: u64, brightness: u32, theme: String) -> MyResult {
        let response = format!(
            "Settings updated for user {}: Theme is now '{}' at {}% brightness.",
            user_id, theme, brightness
        );
        MyResult(Ok(response))
    }
}
```

#### The `#[rest]` Attribute

The `#[rest]` attribute maps your pure Rust function to an HTTP endpoint, giving you full control over the REST API. It has several parts:

* **`method = GET`**: (Required) The HTTP method (`GET`, `POST`, `PUT`, etc.).
* **`path = "/..."`**: (Required) The URL path.
    * Path parameters like `/{user_id}` are automatically mapped to function arguments with the same name (e.g., `user_id: u64`).
* **`query(...)`**: (Optional) A group that lists function arguments to be extracted from the URL's query string.
    * `query(limit)` is shorthand for `query(limit = limit)`.
    * `query(q = search_query)` maps the public query key `q` to the Rust variable `search_query`.
* **`body(...)`**: (Optional) A group that lists function arguments to be bundled into a single JSON object for the request body.
    * `body(brightness, theme)` tells the macro to expect a JSON body like `{"brightness": 85, "theme": "dark"}`.
* **`form(...)`**: (Optional) A group that lists function arguments to be deserialized from a URL-encoded form submission (Content-Type: application/x-www-form-urlencoded).
    * `form(username, password)` expects a form body like `username=alice&password=secret`


### 2. Run the Servers

In your server's binary, use the `ServerBuilder` to launch all protocol endpoints.

```rust
// In your server binary (e.g., src/main.rs)
use example_server_lib::{greeter_impls, MyGreeter}; // Replace with your lib name
use multi_rpc::prelude::*;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let service = MyGreeter("Chauncey".to_string());

    let server_runner = ServerBuilder::new(service)
        .add_protocol(greeter_impls::tarpc_tcp(([127, 0, 0, 1], 9001).into()))
        .add_protocol(greeter_impls::rest_axum(([127, 0, 0, 1], 9002).into()))
        .add_protocol(greeter_impls::jsonrpsee(([127, 0, 0, 1], 9003).into()))
        .build()?;

    server_runner.run().await?;
    Ok(())
}
```

### 3. Calling the Service (Clients)

Once the server is running, you can call its methods from clients for each protocol.

#### Tarpc Client

The `#[multi_rpc_trait]` macro generates a typed client (`GreeterTarpcClient`).

```rust
use example_server_lib::GreeterClient;
use tarpc::{client, context, tokio_serde::formats::Json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect("127.0.0.1:9001", Json::default).await?;
    let client = GreeterClient::new(client::Config::default(), transport).spawn();

    let greet_response = client.greet(context::current(), "Sally".to_string()).await?;
    println!("✅ Tarpc Greet Response: {:?}", greet_response);

    let settings_response = client
        .update_settings(context::current(), 101, 85, "dark".to_string())
        .await?;
    println!("✅ Tarpc Settings Response: {:?}", settings_response);

    Ok(())
}
```

#### REST (reqwest) Client

The REST endpoint is called using a standard HTTP client.

```rust
use example_server_lib::MyResult;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = "[http://127.0.0.1:9002](http://127.0.0.1:9002)";

    // Call the GET endpoint
    let greet_response = client
        .get(format!("{}/greet/sammy", base_url))
        .send().await?.json::<MyResult>().await?;
    println!("✅ REST Greet Response: {:?}", greet_response);

    // Call the POST endpoint with a JSON body
    let settings_body = serde_json::json!({
        "brightness": 85,
        "theme": "dark"
    });
    let update_response = client
        .post(format!("{}/users/101/settings", base_url))
        .json(&settings_body)
        .send().await?.json::<MyResult>().await?;
    println!("✅ REST Settings Response: {:?}", update_response);

    Ok(())
}
```

#### JSON-RPC (jsonrpsee) Client

The JSON-RPC endpoint can be called using positional parameters.

```rust
use example_server_lib::MyResult;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "[http://127.0.0.1:9003](http://127.0.0.1:9003)";
    let client = HttpClientBuilder::default().build(url)?;

    // Call the 'greet' method
    let greet_params = rpc_params!["Jimmy"];
    let greet_response: MyResult = client.request("greet", greet_params).await?;
    println!("✅ JSON-RPC Greet Response: {:?}", greet_response);

    // Call the 'update_settings' method
    let settings_params = rpc_params![101, 85, "dark"];
    let settings_response: MyResult = client.request("update_settings", settings_params).await?;
    println!("✅ JSON-RPC Settings Response: {:?}", settings_response);

    Ok(())
}
```

## Future Plans

### separate rpc from transport

In its initial version, `multi-rpc` conflates the RPC protocol with a specific transport (e.g., Tarpc is tied to TCP, and others are tied to HTTP). This design was chosen for simplicity but lacks flexibility.

A major goal for a future release is to decouple these concepts, allowing users to mix and match protocols with different underlying transports.

### Other possibilities:

* extend RpcError type with more variants.  perhaps allow for custom error types.
* proper logging.  perhaps add optional dep on tracing, or support a logging callback.
* Enhance the #[rest] macro to support different kinds of arguments, such as JSON request bodies (axum::Json) in addition to the currently supported path parameters (axum::Path).
* add support for more protocols (e.g., gRPC, Thrift, Cap'n Proto).
* add support for streaming RPCs.
* add a test framework

## Contributing

Contributions are welcome! In particular, **Pull Requests to add support for new RPC protocols are encouraged**. If you have a protocol you'd like to see supported, please feel free to open an issue or submit a PR.