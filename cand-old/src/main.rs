#[macro_use]
extern crate clap;

mod backend;
mod config;
mod hook;
mod listen;
mod reactor;
mod util;

use crate::config::Backend;
use anyhow::Context;
use std::fs;
use tokio::task;

fn args() -> clap::App<'static, 'static> {
    clap::app_from_crate!().arg(
        clap::Arg::with_name("config")
            .default_value("/etc/cand.toml")
            .short("-c")
            .long("--config")
            .takes_value(true)
            .help("The path of the config file"),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = args().get_matches();

    let config: config::Config = toml::from_slice(
        &fs::read(args.value_of("config").unwrap()).context("Failed to open config file")?,
    )
    .context("Failed to parse config file")?;

    let (stream, sink, task) = match config.backend {
        Backend::SocketCAN { interface } => {
            log::info!("Connecting to CAN interface {}", interface);
            backend::socketcan::connect(&interface)?
        }
        Backend::Network { .. } => {
            todo!("Not yet implemented")
        }
    };

    let (mut reactor, mut handle) = reactor::Reactor::new();

    handle.register_uplink(stream, sink, task).await;

    for listen in config.listen {
        match listen {
            config::Listen::TCP { bind } => {
                log::info!("Listening on TCP {}", bind);
                task::spawn(listen::tcp::listen(bind, handle.clone()));
            }
        }
    }

    {
        let count = config.hooks.len();
        if count == 1 {
            log::info!("Loaded 1 hook");
        } else {
            log::info!("Loaded {} hooks", count);
        }
        hook::hook_task(handle.clone(), config.hooks).await;
    }

    reactor.run().await;

    Ok(())
}
