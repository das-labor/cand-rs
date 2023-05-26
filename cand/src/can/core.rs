use slotmap::DenseSlotMap;
use tokio::{
    sync::mpsc::{self, error::SendTimeoutError},
    task,
};

use super::{CANCommand, CANMessage};
use tokio::time::Duration;

pub async fn spawn(
    backend_config: crate::config::CANBackend,
) -> anyhow::Result<mpsc::Sender<CANCommand>> {
    let (tx, rx) = mpsc::channel(16);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        match run(tx_clone, rx, backend_config).await {
            Ok(()) => {}
            Err(e) => {
                log::error!("Error in Backend Task: {:?}", e)
            }
        }
        log::error!("Backend thread exited");
    });
    Ok(tx)
}

async fn run(
    backend_tx: mpsc::Sender<CANCommand>,
    mut rx: mpsc::Receiver<CANCommand>,
    backend_config: crate::config::CANBackend,
) -> anyhow::Result<()> {
    let (can_tx, backend_rx) = mpsc::channel(16);
    task::spawn(async move {
        match run_backend(backend_config, backend_tx, backend_rx).await {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    });
    let mut subscriptions = DenseSlotMap::new();
    while let Some(msg) = rx.recv().await {
        match msg {
            CANCommand::Subscribe {
                subscription,
                sender,
            } => {
                subscriptions.insert((subscription, sender));
            }
            CANCommand::SendMessage { message } => can_tx.send(message).await.unwrap(),
            CANCommand::ReceiveMessage { message } => {
                let mut broken_keys = vec![];
                for (handle, (subscription, sender)) in &subscriptions {
                    if subscription.matches(&message) {
                        match sender
                            .send_timeout(message, Duration::from_millis(100))
                            .await
                        {
                            Ok(_) => {}
                            Err(SendTimeoutError::Closed(_)) => {
                                broken_keys.push(handle);
                            }
                            Err(SendTimeoutError::Timeout(_)) => {
                                log::warn!("Failed to send CAN subscription {:?} because the queue was full", subscription)
                            }
                        }
                    }
                }
                for k in broken_keys {
                    subscriptions.remove(k);
                }
            }
        }
    }

    Ok(())
}

async fn run_backend(
    config: crate::config::CANBackend,
    _tx: mpsc::Sender<CANCommand>,
    _rx: mpsc::Receiver<CANMessage>,
) -> anyhow::Result<()> {
    match config {
        #[cfg(feature = "backend-network-legacy")]
        crate::config::CANBackend::LegacyNetwork { server } => {
            todo!()
        }
        #[cfg(feature = "backend-network")]
        crate::config::CANBackend::Network { server } => todo!(),
        #[cfg(feature = "backend-interface")]
        crate::config::CANBackend::Interface { interface } => todo!(),
    }
}
