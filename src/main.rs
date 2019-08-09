#![feature(async_await)]
#![feature(async_closure)]

use byteorder::{ByteOrder, NetworkEndian};
use futures::prelude::*;
use runtime::net::{TcpListener, TcpStream};
use std::io;
use std::net::Shutdown;

#[runtime::main]
async fn main() -> io::Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:15000")?;
    println!("listening on {}", listener.local_addr()?);

    listener
        .incoming()
        .try_for_each_concurrent(None, async move |mut stream| {
            runtime::spawn(async move {
                println!("accepting from: {}", stream.peer_addr()?);
                handshake(&mut stream).await?;
                stream.shutdown(Shutdown::Both)?;
                Ok::<(), io::Error>(())
            })
            .await
        })
        .await
}

async fn handshake(stream: &mut TcpStream) -> io::Result<()> {
    let mut client_handshake = [0u8; 4];
    stream.read_exact(&mut client_handshake).await?;
    if &client_handshake != b"\0\0\0\0" {
        println!("incorrect handshake");
        return Ok(());
    }

    let mut server_handshake = [0u8; 4];
    NetworkEndian::write_u32(&mut server_handshake, 42);
    stream.write_all(&server_handshake).await?;

    Ok(())
}
