use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (mut stream, peer) = listener.accept().await?;
        tokio::spawn(async move {
            println!("New connection from {}", peer);
            let mut buf = vec![0u8; 16 * 1024];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        println!("{} closed", peer);
                        break;
                    }
                    Ok(n) => {
                        // Print hex + utf8-ish preview
                        println!("--- {} bytes from {} ---", n, peer);
                        println!("{:02x?}", &buf[..n]);
                        match std::str::from_utf8(&buf[..n]) {
                            Ok(s) => println!("UTF-8:\n{}\n", s),
                            Err(_) => println!("(non-UTF-8)\n"),
                        }
                    }
                    Err(e) => {
                        eprintln!("Read error from {}: {}", peer, e);
                        break;
                    }
                }
            }
        });
    }
}
