use axum::Router;
use axum::response::Redirect;
use lib_api_router::api_router;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

#[cfg(not(feature = "debug-web"))]
use lib_web_router::web_router;

#[cfg(not(feature = "debug-web"))]
fn router() -> Router {
    Router::new()
        .nest_service("/api", api_router())
        .nest_service("/web", web_router())
        .fallback(|| async { Redirect::temporary("/web") })
}

#[cfg(feature = "debug-web")]
fn router() -> Router {
    Router::new()
        .nest_service("/api", api_router())
        .fallback(|| async { Redirect::temporary("http://127.0.0.1:4000/web") })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // run it with hyper on 0.0.0.0:3000
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, router()).await?;

    Ok(())
}
