use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

fn parse_cli_args() -> (String, SocketAddr) {
    let mut args = std::env::args();
    args.next().unwrap();
    let server_addr = args
        .next()
        .expect("server address must be provided as the first CLI argument")
        .parse()
        .expect("server addr must be a correct IP:port address");
    let username = args
        .next()
        .expect("username must be provided as the second CLI argument");
    (username, server_addr)
}

fn reader_thread_main(mut stream: TcpStream) {
    loop {
        let common::Message { sender, text } =
            common::read_message(&mut stream).expect("Message is received");
        const BELL: char = '\x07';
        const ERASE_IN_LINE: &str = "\x1b[0K";
        print!(
            "\r{bell}< {}: {}{el}\n> ",
            sender,
            text,
            bell = BELL,
            el = ERASE_IN_LINE
        );
        std::io::stdout().flush().expect("Flush stdout");
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    let (username, server_addr) = parse_cli_args();

    let mut stream =
        std::net::TcpStream::connect_timeout(&server_addr, CONNECT_TIMEOUT)
            .expect("Connection is established");

    {
        let stream = stream.try_clone().expect("Stream is cloned");
        std::thread::spawn(move || {
            reader_thread_main(stream);
        });
    }

    let input = std::io::stdin();
    let mut output = std::io::stdout();
    loop {
        let mut buf = String::new();
        print!("> ");
        output.flush().expect("Flush stdout");
        input.read_line(&mut buf).expect("Read line from stdin");
        buf = buf.trim_end().to_string();
        common::write_message(
            common::Message {
                sender: username.clone(),
                text: buf,
            },
            &mut stream,
        )
        .expect("Message is sent");
    }
}
