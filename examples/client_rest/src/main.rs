// multi-rpc/examples/client_rest/src/main.rs

use example_server_lib::MyResult;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = "http://127.0.0.1:9002";

    // 1. Call the simple GET endpoint with a path parameter
    println!("--- Calling GET /greet/sammy ---");
    let greet_response = client
        .get(format!("{}/greet/sammy", base_url))
        .send()
        .await?
        .json::<MyResult>()
        .await?;
    println!("✅ GET Response: {:?}\n", greet_response);

    // 2. Call the POST endpoint with a path parameter and a JSON body
    println!("--- Calling POST /users/101/settings ---");
    let settings_body = serde_json::json!({
        "brightness": 85,
        "theme": "dark"
    });

    let update_response = client
        .post(format!("{}/users/101/settings", base_url))
        .json(&settings_body) // This serializes the map to a JSON body
        .send()
        .await?
        .json::<MyResult>()
        .await?;
    println!("✅ POST Response: {:?}", update_response);

    Ok(())
}
