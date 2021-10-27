mod backend;
mod reactor;
mod listen;
mod util;

use tokio::task;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let (mut stream, sink, task) = backend::socketcan::connect("vcan0")?;

    let (mut reactor, mut handle) = reactor::Reactor::new();

    handle.register_uplink(stream, sink, task).await;

    task::spawn(listen::tcp::listen("[::]:2342".parse().unwrap(), handle));

    reactor.run().await;

    Ok(())
}
