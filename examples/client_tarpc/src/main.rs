use example_server_lib::GreeterClient;
use tarpc::client;
use tarpc::context;
use tarpc::tokio_serde::formats::Json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect("127.0.0.1:9001", Json::default).await?;
    let client = GreeterClient::new(client::Config::default(), transport).spawn();

    let greet_response = client
        .greet(context::current(), "Sally".to_string())
        .await?;
    println!("✅ Tarpc Greet Response: {:?}", greet_response);

    let settings_response = client
        .update_settings(context::current(), 101, 85, "dark".to_string())
        .await?;
    println!("✅ Tarpc Settings Response: {:?}", settings_response);

    Ok(())
}
