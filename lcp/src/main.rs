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
            clap::Command::new("list")
                .arg(clap::Arg::new("device").action(ArgAction::Set))
                .arg(clap::Arg::new("room").action(ArgAction::Set)),
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
        .subcommand(
            clap::Command::new("get")
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
        ("list", submatches) => {
            let device_filter = submatches.get_one::<String>("device");
            let room_filter = submatches.get_one::<String>("room");

            let (_rooms, devices) = con.list_devices().await?;

            let filtered_devices = devices
                .into_iter()
                .map(|device| {
                    let device_id = device.id.clone();
                    let channels = device
                        .channels
                        .into_iter()
                        .filter(move |channel| {
                            let dev_ok = if let Some(device_filter) = &device_filter {
                                device_filter.as_bytes() == &device_id
                            } else {
                                true
                            };

                            let room_ok = if let Some(room_filter) = &room_filter {
                                room_filter.as_bytes() == &channel.room
                            } else {
                                true
                            };

                            dev_ok && room_ok
                        })
                        .collect();
                    lcp_client::proto::DeviceDescriptor { channels, ..device }
                })
                .filter(|device| !device.channels.is_empty())
                .collect::<Vec<_>>();

            for device in filtered_devices {
                println!(
                    "{} (ID {})",
                    device.display_name,
                    String::from_utf8_lossy(&device.id)
                );
                println!("    Wiki URL: {}", device.wiki_url);
                println!();

                for channel in device.channels {
                    println!(
                        "    {}:{}:{} {} [{}]",
                        String::from_utf8_lossy(&device.id),
                        String::from_utf8_lossy(&channel.room),
                        String::from_utf8_lossy(&channel.id),
                        channel.display_name,
                        channel.flags,
                    )
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
