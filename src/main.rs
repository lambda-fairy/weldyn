#![feature(async_await)]

use futures::prelude::*;
use runtime::net::TcpListener;

#[runtime::main]
async fn main() -> std::io::Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:15000")?;
    println!("listening on {}", listener.local_addr()?);

    listener
        .incoming()
        .try_for_each_concurrent(None, async move |stream| {
            runtime::spawn(async move {
                println!("accepting from: {}", stream.peer_addr()?);

                let (reader, writer) = &mut stream.split();
                reader.copy_into(writer).await?;

                Ok::<(), std::io::Error>(())
            })
            .await
        })
        .await
}
