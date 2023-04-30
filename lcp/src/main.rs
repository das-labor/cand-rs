#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (con, task) = lcp_client::Connection::connect("[::1]:2342").await?;

    tokio::task::spawn(async move {
        match task.run().await {
            Ok(()) => {}
            Err(e) => {
                println!("Error: {e:?}")
            }
        }
    });

    println!("Devices: {:#?}", con.list_devices().await?);

    Ok(())
}
