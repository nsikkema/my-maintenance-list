use crate::wide_event::WideEvent;
use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct WideEventLoggerLayer<F> {
    sampling_enabled: bool,
    print_fn: F,

    keys: HashMap<String, String>,
}

impl<F> WideEventLoggerLayer<F>
where
    F: Fn(String) + Send + Sync + Clone + 'static,
{
    pub fn new(sampling_enabled: bool, print_fn: F, keys: HashMap<String, String>) -> Self {
        Self {
            sampling_enabled,
            print_fn,

            keys,
        }
    }
}

impl<S, F> Layer<S> for WideEventLoggerLayer<F>
where
    F: Fn(String) + Send + Sync + Clone + 'static,
{
    type Service = WideEventLogger<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        WideEventLogger {
            inner,

            sampling_enabled: self.sampling_enabled,
            print_fn: self.print_fn.clone(),

            keys: self.keys.clone(),
        }
    }
}

#[derive(Clone)]
pub struct WideEventLogger<S, F> {
    inner: S,

    sampling_enabled: bool,
    print_fn: F,

    keys: HashMap<String, String>,
}

impl<S, F> Service<Request> for WideEventLogger<S, F>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    F: Fn(String) + Send + Sync + Clone + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let sampling_enabled = self.sampling_enabled;
        let start = Instant::now();

        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let method = request.method().to_string();
        let path = request.uri().path().to_string();

        let event = Arc::new(Mutex::new(WideEvent::new(
            request_id,
            method,
            path,
            self.keys.clone(),
        )));

        request.extensions_mut().insert(Arc::clone(&event));

        let mut inner = self.inner.clone();
        let print_fn = self.print_fn.clone();

        Box::pin(async move {
            let response = inner.call(request).await?;

            let status = response.status();
            let mut event_lock = event.lock().expect("Failed to lock wide event");

            event_lock.status_code = status.as_u16();
            event_lock.duration_ms = start.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

            if event_lock.should_sample(sampling_enabled) {
                (print_fn)(serde_json::to_string(&*event_lock).unwrap_or_default());
            }

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_custom_print_fn() {
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        let print_fn = move |_s: String| {
            called_clone.store(true, Ordering::SeqCst);
        };

        let layer = WideEventLoggerLayer::new(
            false, // sampling disabled (log everything)
            print_fn,
            HashMap::new(),
        );

        let mut service = layer.layer(tower::service_fn(|_req: Request| async {
            Ok::<Response, std::convert::Infallible>(Response::new(Body::empty()))
        }));

        let req = HttpRequest::builder().uri("/").body(Body::empty()).unwrap();

        let _res = service.ready().await.unwrap().call(req).await.unwrap();

        assert!(called.load(Ordering::SeqCst));
    }
}
