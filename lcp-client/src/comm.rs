use lcp_proto::{Message, ToClientPayload, ToServerPayload};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum ToBackend {
    SendRequest {
        reply: mpsc::Sender<ToFrontend>,
        payload: ToServerPayload,
        multi_response: bool,
    },
    #[allow(dead_code)]
    Unregister {
        req_id: u64,
    },
    MessageReceived {
        message: Message<ToClientPayload>,
    },
}

#[derive(Debug)]
pub enum ToFrontend {
    RequestSent { req_id: u64 },
    Response(ToClientPayload),
}
