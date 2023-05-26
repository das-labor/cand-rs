#[cfg(any(feature = "backend-network-legacy", feature = "backend-network"))]
use std::net::SocketAddr;

pub enum CANBackend {
    #[cfg(feature = "backend-network-legacy")]
    LegacyNetwork { server: SocketAddr },
    #[cfg(feature = "backend-network")]
    Network { server: SocketAddr },
    #[cfg(feature = "backend-interface")]
    Interface { interface: String },
}
