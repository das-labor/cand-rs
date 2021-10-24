mod backend;
mod reactor;
mod listen;
mod util;

use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let (mut stream, _sink, _handle) = backend::socketcan::connect("vcan0")?;
    while let Some(element) = stream.next().await {
        println!("Message: {:?}", element);
    }
    Ok(())
}
