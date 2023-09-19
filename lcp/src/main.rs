use clap::ArgAction;

fn args() -> clap::Command {
    clap::command!()
        .subcommand_required(true)
        .arg(
            clap::Arg::new("server")
                .short('s')
                .long("server")
                .action(ArgAction::Set)
                .help("The cand to connect to")
                .default_value("cand:2342"),
        )
        .subcommand(
            clap::Command::new("set")
                .arg(
                    clap::Arg::new("device")
                        .required(true)
                        .action(ArgAction::Set),
                )
                .arg(clap::Arg::new("room").required(true).action(ArgAction::Set))
                .arg(
                    clap::Arg::new("channel")
                        .required(true)
                        .action(ArgAction::Set),
                )
                .arg(
                    clap::Arg::new("value")
                        .required(true)
                        .action(ArgAction::Set),
                ),
        )
        .subcommand(clap::Command::new("show-channels"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let matches = args().get_matches();

    let (con, task) =
        lcp_client::Connection::connect(&matches.get_one::<String>("server").unwrap()).await?;

    tokio::task::spawn(async move {
        match task.run().await {
            Ok(()) => {}
            Err(e) => {
                println!("Error: {e:?}")
            }
        }
    });

    match matches.subcommand().unwrap() {
        ("show-channels", _) => {
            println!("Devices: {:#?}", con.list_devices().await?);
        }
        ("set", submatches) => {
            con.set_channel(
                submatches.get_one::<String>("device").unwrap().as_bytes(),
                submatches.get_one::<String>("room").unwrap().as_bytes(),
                submatches.get_one::<String>("channel").unwrap().as_bytes(),
                lcp_client::Value::Text(submatches.get_one::<String>("value").unwrap().to_owned()),
            )
            .await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
