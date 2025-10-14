// multi-rpc/examples/server/src/lib.rs

use multi_rpc::error::RpcError;
use multi_rpc::*;

// Demonstrates that we can return a custom type.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MyResult(pub Result<String, RpcError>);

#[multi_rpc_trait]
trait Greeter {
    /// A simple method that takes a path parameter.
    async fn greet(&self, name: String) -> MyResult;

    /// A more complex method that mixes a path parameter and a multi-field JSON body.
    async fn update_settings(&mut self, user_id: u64, brightness: u32, theme: String) -> MyResult;
}

#[derive(Clone)]
pub struct MyGreeter(pub String);

#[multi_rpc_impl]
impl Greeter for MyGreeter {
    // This method only has a path parameter, which is inferred from the path string.
    #[rest(method = GET, path = "/greet/{name}")]
    async fn greet(&self, name: String) -> MyResult {
        println!("[greet] Received call for name: {}", name);
        if name.is_empty() {
            return MyResult(Err(RpcError::InternalError(
                "Name cannot be empty".to_string(),
            )));
        }
        MyResult(Ok(format!("Hello, {}! My name is {}.", name, self.0)))
    }

    // This method has one path parameter (`user_id`) and two body parameters.
    // The `body(...)` group lists the arguments to be bundled into the JSON body.
    #[rest(method = POST, path = "/users/{user_id}/settings", body(brightness, theme))]
    async fn update_settings(&mut self, user_id: u64, brightness: u32, theme: String) -> MyResult {
        println!(
            "[update_settings] Received for user_id {}: brightness={}, theme='{}'",
            user_id, brightness, theme
        );
        let response = format!(
            "Settings updated for user {}: Theme is now '{}' at {}% brightness.",
            user_id, theme, brightness
        );
        MyResult(Ok(response))
    }
}
