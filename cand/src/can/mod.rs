pub mod core;
#[cfg(feature = "backend-network-legacy")]
pub mod legacy;

use std::io;

use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug)]
pub struct CANSubscription {
    id: u16,
    id_mask: u16,
    payload: [u8; 8],
    payload_mask: [u8; 8],
    min_len: usize,
    max_len: usize,
}

impl CANSubscription {
    pub fn build() -> CANSubscriptionBuilder {
        CANSubscriptionBuilder {
            id: 0,
            id_mask: 0,
            payload: [0u8; 8],
            payload_mask: [0u8; 8],
            min_len: 0,
            max_len: 8,
        }
    }

    pub fn matches(&self, msg: &CANMessage) -> bool {
        and_buf(&msg.payload, &self.payload_mask) == self.payload
            && msg.id & self.id_mask == self.id
            && msg.len >= self.min_len
            && msg.len <= self.max_len
    }
}

pub struct CANSubscriptionBuilder {
    id: u16,
    id_mask: u16,
    payload: [u8; 8],
    payload_mask: [u8; 8],
    min_len: usize,
    max_len: usize,
}

impl CANSubscriptionBuilder {
    fn match_id(mut self, id: u16, mask: u16) -> Self {
        if id > 0x07ff || mask > 0x07ff {
            panic!("Invalid CAN ID / Mask");
        }
        self.id = id & mask;
        self.id_mask = mask;
        self
    }

    fn match_payload(mut self, payload: [u8; 8], mask: [u8; 8]) -> Self {
        self.payload = and_buf(&payload, &mask);
        self.payload_mask = mask;
        self
    }

    fn min_len(mut self, len: u8) -> Self {
        let len = len as usize;
        if len > 8 {
            panic!("CAN Messages cannot be larger than 8 bytes")
        }
        if len > self.max_len {
            panic!("Minimum length cannot be higher than maximum")
        }
        self.min_len = len;
        self
    }

    fn max_len(mut self, len: u8) -> Self {
        let len = len as usize;
        if len > 8 {
            panic!("CAN Messages cannot be larger than 8 bytes")
        }
        if len < self.min_len {
            panic!("Minimum length cannot be higher than maximum")
        }
        self.max_len = len;
        self
    }

    fn build(self) -> CANSubscription {
        CANSubscription {
            id: self.id,
            id_mask: self.id_mask,
            payload: self.payload,
            payload_mask: self.payload_mask,
            min_len: self.min_len,
            max_len: self.max_len,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CANMessage {
    id: u16,
    payload: [u8; 8],
    len: usize,
}

impl CANMessage {
    pub fn read<R: io::Read>(_read: &mut R) -> io::Result<Self> {
        unimplemented!()
    }

    pub fn write<W: io::Write>(&self, _write: &mut W) -> io::Result<()> {
        unimplemented!()
    }
}

pub enum CANCommand {
    /// Subscribe to Certain CAN events. THe subscription field specifies which events to react to
    /// The respective messages will be sent via the receiver
    Subscribe {
        subscription: CANSubscription,
        sender: mpsc::Sender<CANMessage>,
    },
    /// Sends a message to the CAN Bus, this can be either a reply or any other kind of message
    SendMessage { message: CANMessage },
    /// Tells the core about a new CAN message that has been received. Only used internally
    ReceiveMessage { message: CANMessage },
}

fn and_buf(a: &[u8; 8], b: &[u8; 8]) -> [u8; 8] {
    let mut ret = [0u8; 8];
    for (i, (a, b)) in a.into_iter().zip(b.into_iter()).enumerate() {
        ret[i] = a & b;
    }
    ret
}
