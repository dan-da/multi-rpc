use example_server_lib::greeter_generated::GreeterTarpcClient; // For tarpc client
use tarpc::client;
use tarpc::tokio_serde::formats::Json;
use tarpc::context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = tarpc::serde_transport::tcp::connect("127.0.0.1:9001", Json::default)
        .await?;

    let client = GreeterTarpcClient::new(client::Config::default(), transport).spawn();
    let response = client
        .greet(tarpc::context::current(), "Sally".to_string())
        .await?;
    println!("✅ Tarpc Response: {:?}", response);

    let settings_response = client.update_settings(context::current(), 101, 85, "dark".to_string()).await?;
    println!("✅ Tarpc Settings Response: {:?}", settings_response);

    Ok(())
}
