use example_server_lib::jsonrpsee;
use example_server_lib::rest_axum;
use example_server_lib::tarpc_tcp;
use example_server_lib::MyGreeter;
use multi_rpc::builder::ServerBuilder;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let service = MyGreeter("Chauncey".to_string());

    let server_runner = ServerBuilder::new(service)
        .add_protocol(tarpc_tcp(([127, 0, 0, 1], 9001).into()))
        .add_protocol(rest_axum(([127, 0, 0, 1], 9002).into()))
        .add_protocol(jsonrpsee(([127, 0, 0, 1], 9003).into()))
        .build()?;

    server_runner.run().await?;

    Ok(())
}
