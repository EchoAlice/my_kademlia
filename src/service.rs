use tokio::sync::mpsc;

pub struct Service {
    // service_rx:
    // pub socket: Arc<UdpSocket>,
    // pub outbound_requests: HashMap<Identifier, (Message, mpsc::recieve<bool>)>,
}

impl Service {
    pub fn spawn() {
        // TODO: Bind UDPSocket here.
    }
    pub fn start(&self) {}
}
