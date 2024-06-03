use std::error::Error;
use std::time::Duration;

use bytes::Bytes;
use h2::{client, Reason};
use http::{Method, Request};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tcp = TcpStream::connect("127.0.0.1:4034").await?;
    let (h2, connection) = client::handshake(tcp).await?;
    tokio::spawn(async move {
        connection.await.unwrap();
    });
    let mut send_request = h2.ready().await?;

    let request = Request::get("http://127.0.0.1:4034")
        .method(Method::POST)
        .body(())
        .unwrap();

    let (response, mut send_stream) = send_request
        .send_request(request, false).unwrap();

    send_stream.send_data(Bytes::from(vec![5; 1000]), false)?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    send_stream.send_reset(Reason::CANCEL);
    let response = response.await?;
    println!("{:?}", response);
    Ok(())
}
