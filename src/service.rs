use tokio::sync::mpsc;

type Channel<T> = mpsc::Receiver<T>;
pub struct Service {
    node_rx: Channel<bool>, // TODO: Channel<Message>
                            // pub socket: Arc<UdpSocket>,
                            // pub outbound_requests: HashMap<Identifier, (Message, mpsc::recieve<bool>)>,
}

impl Service {
    pub fn spawn() -> mpsc::Sender<bool> {
        let (tx, node_rx) = mpsc::channel(32);

        // TODO: Bind UDPSocket here.

        let mut service = Service { node_rx };

        println!("Spawning service");

        // Create loop that listens for a bool
        tokio::spawn(async move {
            service.start().await;
        });

        tx
    }

    // Create loop that listens for a bool
    pub async fn start(&mut self) {
        // TODO: Should I implement tokio::select!  ???
        loop {
            match self.node_rx.recv().await {
                Some(true) => {
                    println!("True was sent through channel to service");
                }
                Some(false) => {
                    println!("False was sent through channel to service");
                }
                _ => {}
            }
        }
    }
}
