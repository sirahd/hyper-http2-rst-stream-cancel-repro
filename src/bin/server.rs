use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::{Method, Request, Response};
use hyper::body::Bytes;
use hyper::rt::Executor;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn hello(mut request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("receiving request {:?}", request);
    match *request.method() {
        Method::POST => {
            let data = request.body_mut().collect().await.unwrap().to_bytes();
            let transfer_bytes = data.len();

            println!("{:?}", transfer_bytes);
            Ok(Response::new(Full::new(Bytes::from(transfer_bytes.to_string()))))
        }
        _ => Ok(Response::new(Full::new(Bytes::new())))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4034));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http2::Builder::new(TokioExecutor)
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[derive(Clone)]
struct TokioExecutor;

impl<F> Executor<F> for TokioExecutor
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
{
    fn execute(&self, future: F) {
        tokio::spawn(future);
    }
}
