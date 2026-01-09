use axum::Router;
use axum::response::Redirect;
use lib_middleware_wide_events::wide_event_layer::WideEventLoggerLayer;
use lib_router_api::api_router;
use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

#[cfg(not(feature = "debug-web"))]
use lib_router_web::web_router;

fn router() -> Router {
    let sampling_enabled = env::var("WIDE_EVENT_SAMPLING_ENABLED")
        .map(|v| v.parse().unwrap_or(true))
        .unwrap_or(true);

    let router = Router::new().nest_service("/api", api_router());

    #[cfg(not(feature = "debug-web"))]
    let router = router
        .nest_service("/web", web_router())
        .route(
            "/favicon.ico",
            axum::routing::get(|| async { Redirect::temporary("/web/favicon.ico") }),
        )
        .fallback(|| async { Redirect::temporary("/web") });

    #[cfg(feature = "debug-web")]
    let router = router.fallback(|| async { Redirect::temporary("http://127.0.0.1:4000/web") });

    router.layer(WideEventLoggerLayer::new(
        sampling_enabled,
        |s: String| println!("{}", s),
        None,
        None,
        None,
        None,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // run it with hyper on 0.0.0.0:3000
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, router()).await?;

    Ok(())
}
