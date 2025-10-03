use example_server_lib::greeter_impls;
use example_server_lib::MyGreeter;
use multi_rpc::builder::ServerBuilder;

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
