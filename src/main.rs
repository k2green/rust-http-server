mod http;

use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

const HOST_ADDR_VARIABLE: &str = "HOST_ADDR";

#[tokio::main]
async fn main() {
    let address = get_host_addr();
    let result = tokio::select! {
        res = tokio::spawn(run_console()) => res,
        res = tokio::spawn(run_server(address)) => res,
    };

    result.expect("An error occurred while running the server");
}

fn get_host_addr() -> String {
    if let Some(addr) = std::env::args().nth(1) {
        addr
    } else if let Ok(addr) = std::env::var(HOST_ADDR_VARIABLE) {
        addr
    } else {
        String::from("127.0.0.1:8080")
    }
}

async fn run_console() {
    let stdin = std::io::stdin();

    loop {
        let mut command = String::new();
        if let Ok(_) = stdin.read_line(&mut command) {
            let parts = command
                .split_whitespace()
                .filter(|s| !s.trim().is_empty())
                .collect::<Vec<_>>();

            match parts.get(0).map(|s| *s) {
                Some("quit" | "q" | "stop") => break,
                _ => ()
            }
        }
    }
}

async fn run_server(addr: impl ToSocketAddrs) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            log::error!("Failed to bind TCP listener: {}", e);
            return;
        }
    };

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection_wrapper(stream, addr));
    }
}

async fn handle_connection_wrapper(stream: TcpStream, addr: SocketAddr) {
    if let Err(e) = handle_connection(stream, addr).await {
        log::error!("An error occurred while handling the connection for {}: {}", addr, e);
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
    println!("Connection established with {}", addr);

    stream.readable().await?;
    let message = read_all(&stream)?;
    
    match String::from_utf8(message) {
        Ok(msg) => println!("===== Received message =====\n{}\n============================", msg),
        Err(_) => println!("Received byte message.")
    }

    stream.writable().await?;
    stream.try_write(b"HTTP/1.1 200 OK\r\n\r\n")?;

    println!("Connection with {} closed", addr);

    Ok(())
}

fn read_all(stream: &TcpStream) -> anyhow::Result<Vec<u8>> {
    let mut output_buffer = Vec::new();

    loop {
        let mut temp_buffer = [0_u8; 4096];
        match stream.try_read(&mut temp_buffer) {
            Ok(0) => break,
            Ok(count) => output_buffer.extend_from_slice(&temp_buffer[0..count]),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(e.into())
        }
    }

    Ok(output_buffer)
}
