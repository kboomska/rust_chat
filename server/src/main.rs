use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, SyncSender},
    },
};

use common::Message;

type ClientMap =
    Arc<RwLock<HashMap<std::net::SocketAddr, std::net::TcpStream>>>;

const MESSAGE_CHANNEL_SIZE: usize = 256;

struct Config {
    addr: std::net::SocketAddr,
}

fn reader_thread_main(
    mut stream: TcpStream,
    sender: SyncSender<Message>,
) -> std::io::Result<()> {
    loop {
        let msg = common::read_message(&mut stream)?;

        log::info!(
            "Message received from {}({}): {}",
            msg.sender,
            stream.peer_addr().unwrap(),
            msg.text
        );

        if sender.send(msg).is_err() {
            log::error!(
                "Reader thread failed to submit a message to channel, this thread will be terminated"
            );
            return Err(std::io::ErrorKind::Other.into());
        }
    }
}

fn broadcast_thread_main(
    clients: ClientMap,
    receiver: Receiver<Message>,
) -> std::io::Result<()> {
    loop {
        let msg: Message = receiver.recv().map_err(|_| {
            log::error!(
                "Failed to receive a message in broadcast thread, the thread will be terminated"
            );
            std::io::ErrorKind::Other
        })?;
        log::info!("Broadcasting message {}: {}", msg.sender, msg.text);
        for (addr, client) in
            clients.write().expect("Lock is not poisoned").iter_mut()
        {
            log::debug!(
                "Sending message {}: {} to {}",
                msg.sender,
                msg.text,
                addr
            );
            common::write_message(msg.clone(), client)?;
        }
    }
}

fn run_server(cfg: Config) -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind(cfg.addr)?;

    let clients: ClientMap = Arc::new(RwLock::new(HashMap::new()));

    let (sender, receiver) =
        std::sync::mpsc::sync_channel(MESSAGE_CHANNEL_SIZE);

    {
        let clients = clients.clone();
        std::thread::spawn(move || {
            if let Err(err) = broadcast_thread_main(clients, receiver) {
                log::error!("Broadcast thread failed: {:?}", err);
            }
        });
    }

    loop {
        let (stream, addr) = listener.accept()?;
        log::info!("New client connected: {}", addr);
        clients
            .write()
            .expect("Lock is not poisoned")
            .insert(addr, stream.try_clone()?);

        let sender = sender.clone();
        let clients = clients.clone();
        std::thread::spawn(move || {
            match reader_thread_main(stream, sender) {
                Ok(_) => {
                    log::info!(
                        "Reader thread finished because connection is over"
                    )
                }
                Err(err)
                    if [
                        std::io::ErrorKind::UnexpectedEof,
                        std::io::ErrorKind::ConnectionReset,
                    ]
                    .contains(&err.kind()) =>
                {
                    log::info!(
                        "Reader thread finished because connection is over (EOF)"
                    )
                }
                Err(err) => {
                    log::error!("Reader thread failed: {:?}", err)
                }
            }
            log::info!("Disconnect from {}", addr);
            clients.write().expect("Lock is not poisoned").remove(&addr);
        });
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    run_server(Config {
        addr: "0.0.0.0:4242".parse().unwrap(),
    })
    .unwrap();
}
