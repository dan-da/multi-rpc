use example_server_lib::MyResult;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = "127.0.0.1:9003";
    let url = format!("http://{}", server_addr);

    let client = HttpClient::builder().build(url)?;
    let params = rpc_params!["Jimmy"];
    let response: MyResult = client.request("greet", params).await?;
    println!("✅ JSON-RPC Response: {:?}", response);

    // Call the 'update_settings' method
    let settings_params = rpc_params![101, 85, "dark"];
    let settings_response: MyResult = client.request("update_settings", settings_params).await?;
    println!("✅ JSON-RPC Settings Response: {:?}", settings_response);

    Ok(())
}
